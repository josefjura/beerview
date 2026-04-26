use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;

#[derive(Parser)]
#[command(
    name = "beerview-admin",
    about = "Beerview administration CLI",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new pub
    CreatePub {
        /// URL-safe slug (e.g. tvujpivnibar)
        #[arg(long)]
        slug: String,

        /// Display name
        #[arg(long)]
        name: String,

        /// Neighbourhood / district (optional)
        #[arg(long)]
        neighbourhood: Option<String>,

        /// Number of taps
        #[arg(long, default_value = "4")]
        taps: i64,
    },

    /// Create a user account for an existing pub
    CreateUser {
        /// Slug of the pub this user belongs to
        #[arg(long)]
        pub_slug: String,

        /// Login username
        #[arg(long)]
        username: String,

        /// Initial password (will be hashed)
        #[arg(long)]
        password: String,

        /// Require password change on first login
        #[arg(long, default_value = "true")]
        must_change_password: bool,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./beerview.db".to_string());

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error: could not connect to database at {database_url}: {e}");
            std::process::exit(1);
        });

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error: migration failed: {e}");
            std::process::exit(1);
        });

    match cli.command {
        Commands::CreatePub { slug, name, neighbourhood, taps } => {
            create_pub(&pool, &slug, &name, neighbourhood.as_deref(), taps).await;
        }
        Commands::CreateUser { pub_slug, username, password, must_change_password } => {
            create_user(&pool, &pub_slug, &username, &password, must_change_password).await;
        }
    }
}

async fn create_pub(
    pool: &sqlx::SqlitePool,
    slug: &str,
    name: &str,
    neighbourhood: Option<&str>,
    tap_count: i64,
) {
    // Validate slug
    if slug.is_empty() || !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        eprintln!("Error: slug must contain only lowercase letters, digits, and hyphens");
        std::process::exit(1);
    }

    let result = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO pub (slug, name, neighbourhood, tap_count) VALUES (?, ?, ?, ?) RETURNING id"
    )
    .bind(slug)
    .bind(name)
    .bind(neighbourhood)
    .bind(tap_count)
    .fetch_one(pool)
    .await;

    let pub_id = match result {
        Ok((id,)) => id,
        Err(e) if e.to_string().contains("UNIQUE") => {
            eprintln!("Error: a pub with slug '{slug}' already exists");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: failed to create pub: {e}");
            std::process::exit(1);
        }
    };

    // Create the tap rows
    for tap_number in 1..=tap_count {
        if let Err(e) = sqlx::query("INSERT INTO tap (pub_id, tap_number) VALUES (?, ?)")
            .bind(pub_id)
            .bind(tap_number)
            .execute(pool)
            .await
        {
            eprintln!("Warning: could not create tap {tap_number}: {e}");
        }
    }

    println!("✓ Pub created");
    println!("  id:            {pub_id}");
    println!("  slug:          {slug}");
    println!("  name:          {name}");
    if let Some(n) = neighbourhood {
        println!("  neighbourhood: {n}");
    }
    println!("  taps:          {tap_count}");
}

async fn create_user(
    pool: &sqlx::SqlitePool,
    pub_slug: &str,
    username: &str,
    password: &str,
    must_change_password: bool,
) {
    // Validate password length
    if password.len() < 8 {
        eprintln!("Error: password must be at least 8 characters");
        std::process::exit(1);
    }

    // Look up the pub
    let pub_row = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM pub WHERE slug=?"
    )
    .bind(pub_slug)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Error: database error: {e}");
        std::process::exit(1);
    });

    let (pub_id, pub_name) = match pub_row {
        Some(row) => row,
        None => {
            eprintln!("Error: no pub found with slug '{pub_slug}'");
            std::process::exit(1);
        }
    };

    // Hash the password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to hash password: {e}");
            std::process::exit(1);
        })
        .to_string();

    let result = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO pub_user (pub_id, username, password_hash, must_change_password)
         VALUES (?, ?, ?, ?) RETURNING id"
    )
    .bind(pub_id)
    .bind(username)
    .bind(&password_hash)
    .bind(must_change_password)
    .fetch_one(pool)
    .await;

    let user_id = match result {
        Ok((id,)) => id,
        Err(e) if e.to_string().contains("UNIQUE") => {
            eprintln!("Error: username '{username}' is already taken");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: failed to create user: {e}");
            std::process::exit(1);
        }
    };

    println!("✓ User created");
    println!("  id:                  {user_id}");
    println!("  username:            {username}");
    println!("  pub:                 {pub_name} ({pub_slug})");
    println!("  must_change_password: {must_change_password}");
}
