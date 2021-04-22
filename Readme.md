# systemd-fail-notifications

This software aims to provide a daemon that can be used to listen on a system bus to systemd changes and react to failed units with notifications.

It is somewhat similar to [systemd_mon](https://github.com/joonty/systemd_mon), but instead of listening on the dbus for changes, polling is used to determine the current state of all systemd units.
The configuration is done via environment variables and can not be set via a configuration file or command line arguments.

It requires a Linux host with systemd installed.

## Deployment

There are two ways available to run this application: either by using the static binary (e.g. as systemd-service) or as a container.

### Container

The easiest way to obtain a current version of this software and run it is to create a container image.
Clone this repository and build the container image by executing `docker build -t systemd-fail-notifications:latest .`.

To run the container (e.g. with docker):

```bash
docker run \
    -v /var/run/dbus/system_bus_socket:/var/run/dbus/system_bus_socket \
    -e SYSTEMD_FAIL_NOTIFICATIONS_DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/<id>/<token>" \
    systemd-fail-notifications:latest
```

### Static Binary

Download the static binary from the [releases page](/localthomas/systemd-fail-notifications/releases).

Create a service file (e.g. `systemd-fail-notifications.service`):

```
[Unit]
Description=Monitoring of failed systemd services

[Service]
Restart=always
Environment="SYSTEMD_FAIL_NOTIFICATIONS_DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/<id>/<token>"
ExecStart=/path/to/systemd-fail-notifications

[Install]
WantedBy=multi-user.target
```

Or specify the environment variables in a [separate file](https://www.freedesktop.org/software/systemd/man/systemd.exec.html#EnvironmentFile=).

Enable the service file by running `systemctl enable /path/to/systemd-fail-notifications.service`.


## Configuration Options

| Name | Format | Description |
| ---- | ------ | ----------- |
| `SYSTEMD_FAIL_NOTIFICATIONS_DISCORD_WEBHOOK_URL` | `https://discord.com/api/webhooks/<id>/<token>` | [Discord webhook URL](https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks) |

#### License

This repository aims to be compliant with the [REUSE specification 3.0](https://reuse.software/spec/).

A list of third-party licenses can be obtained by executing the binary with the `--about` flag.

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSES/Apache-2.0.txt) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSES/MIT.txt) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
