# iCaly

## MVP Feature set

### When a event is created / updated / deleted on Discord we should update the iCal.
- [x] Listen to Discord ScheduledEvent changes with Serenity.
- [ ] Fetch all the SchedueldEvents on startup.
- [ ] Re-fetch all the SchedueldEvents when a ScheduledEvent is modified.
- [ ] Write the fetched SchedueldEvents to a iCal file.
- [ ] Serve the iCal files with Nginx.
- [ ] Persist past events?
- [ ] Ensure data deletion after exiting a server.
- [ ] HTTPS redirect.
