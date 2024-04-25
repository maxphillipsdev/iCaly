# iCaly

## MVP Feature set

### When a event is created / updated / deleted on Discord we should update the iCal.
- [x] Listen to Discord ScheduledEvent changes with Serenity.
- [x] Fetch all the SchedueldEvents on startup.
- [x] Re-fetch all the SchedueldEvents when a ScheduledEvent is modified.
- [x] Write the fetched SchedueldEvents to a iCal file.
- [x] Serve the iCal files with Nginx.
- [x] Ensure data deletion after exiting a server.
- [ ] HTTPS redirect.
