# supertask

[![Checks](https://img.shields.io/github/actions/workflow/status/norskeld/supertask/checks.yml?style=flat-square&colorA=22272d&colorB=22272d&label=checks)](https://github.com/norskeld/supertask/actions/workflows/checks.yml)

Declarative cron-like command orchestration tool.

## Example

Events, along with services and requirements for them, are defined in a single YAML file:

```yaml
events:
  dotfiles:
    run:
      - cd ~/.dotfiles
      - git fetch
      - git pull
    interval: twice per day
    require: network

  backup:
    run:
      - rsync -av ~/Documents some-machine:backups/Documents
      - rsync -av ~/Pictures  other-machine:backups/Pictures
      - rsync -av ~/Downloads remote-machine:backups/Downloads
    parallel: true
    interval: every other day
    require: network

services:
  syncthing:
    run: syncthing

requirements:
  network: nc -zG 1 1.1.1.1 53
```

## Schema

Not really a formal "schema", more like a reference for the format of the configuration file and what can be defined in it.

```yaml
# Events are commands that are run at certain intervals, either sequentially or in parallel.
events:
  event-name:
    run: <command-string>
    interval: [<seconds>|hourly|daily|weekly|...]

  event-name:
    run:
      - <command-string>
      - <command-string>
    interval: [<seconds>|hourly|daily|weekly|...]
    parallel: <bool>

  event-name:
    run: <command-string>
    interval: [<seconds>|hourly|daily|weekly|...]
    require: <requirement-name>
    monitor: <command-string>

# Requirements are simply named commands that are run to determine whether the event should be run # or not. If the command returns a non-zero exit code, the event depending on it will not be run.
requirements:
  requirement-name: <command-string>
  requirement-name: <command-string>

services:
  service-name:
    run: [process name] <args>

  service-name:
    run: [process name] <args>
```

## License

[MIT](LICENSE).
