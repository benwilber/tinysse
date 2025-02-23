-- luacheck: globals json uuid http sleep log url sqlite

-- Preload the modules so they are available to the script
-- via `require` instead of referencing globals.
package.loaded.json = json
package.loaded.uuid = uuid
package.loaded.http = http
package.loaded.sleep = sleep
package.loaded.log = log
package.loaded.url = url
package.loaded.sqlite = sqlite
