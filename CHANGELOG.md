0.6.0 (2025-03-26)
===================
* New `fernet` package in the Lua API
* Add the program/version banner to the startup log

0.5.0 (2025-03-26)
===================
* New `base64` package in the Lua API
* Add docs for the `mutex` and `sleep` Lua built-ins
* Add example for publishing occupancy counts at regular intervals

0.4.0 (2025-03-22)
===================
* Always call `catchup(sub, last_event_id)` even if last_event_id is `nil`

0.3.1 (2025-03-22)
===================
* Add --version flag to CLI and bump version to 0.3.1

0.3.0 (2025-03-22)
===================
* Implement message catch-up via Last-Event-ID

0.2.0 (2025-03-20)
===================
Initial release
