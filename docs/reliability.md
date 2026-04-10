# Reliability & Testing

## The lazy owner failure mode

The target users are, by design, people who will not tolerate friction. A bug or confusing moment during service — the worst possible time — means "I don't need this anymore." There is no second chance. This is not a normal app where a user files a bug report and waits for a fix.

**Consequence:** reliability is not a nice-to-have. It is the product. Everything else — features, design, integrations — is secondary to the app working flawlessly at the moment of a keg switch.

---

## Design principles for resilience

### The Switch action is sacred

The one moment the owner *must* interact with beerview is the keg switch. It happens fast, under pressure, one-handed, mid-conversation. It must:

- Respond instantly (optimistic UI — show the change immediately, sync in background)
- Never show an error dialog during the action itself
- Never require a second tap to confirm
- Have an **undo within ~30 seconds** for misfires — friendlier than a confirmation dialog and doesn't slow down the happy path

### Graceful degradation on external dependencies

Every integration is optional from the core's perspective:

| If this fails... | The app should... |
|---|---|
| Facebook API | Log the failure, retry later, never surface to owner during switch |
| Untappd search | Fall back to manual entry, no drama |
| Instagram API | Same as Facebook |
| Beer database | Allow manual entry |
| Embed widget fetch | Show cached last-known tap list, not an error |

**Rule:** the owner should never see an external API failure during a keg switch. Only during prep time (adding beers to queue) where they have time and patience to deal with it.

### Offline tolerance

Pubs have unpredictable wifi. The Switch action should work optimistically and sync when connectivity returns. The owner should not know or care whether they were online at the moment of the switch.

---

## Testing strategy

### Integration tests — highest priority

The Switch transaction is the most critical code path:
- Beer moves from Queue → On Tap atomically
- Previous On Tap beer moves to Archive
- Public tap list reflects the change immediately
- Social post is queued (not blocking the response)
- All of this survives a network hiccup or server restart mid-transaction

### Chaos / fault injection tests

Simulate external API failures during normal operations:
- Untappd down during beer search
- Facebook API rate-limited or returning 5xx
- Database write succeeds but social post fails — tap list must still update
- Concurrent switches from two devices (owner + staff)

### E2E tests simulating real usage

The full workflow on a real mobile-sized viewport:
- Add beer to queue
- Perform a switch during simulated "busy mode" (slow connection, small screen)
- Verify public tap list updated
- Verify undo works within the window
- Verify social post queued

### Beta testing — the real validation

The three Prague pubs are the best possible test environment. Real conditions, real pressure, real laziness threshold.

**Success criterion:** survives a Friday night in all three pubs for one month without a complaint. If an owner doesn't mention it, it's working.

---

## Monitoring

Even for a free community project, basic observability matters:
- Alert on Switch failures (the sacred action should never silently fail)
- Log external API failures with enough context to diagnose
- Track "last updated" per pub — if a pub goes silent for weeks, something may be wrong

The public discovery site showing "last updated" timestamps is itself a form of monitoring — stale pubs are visible.
