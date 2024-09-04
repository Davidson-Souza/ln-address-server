# Poor-man's lnaddress server

This is a super minimal implementation of lnurl pay and lnaddress to self-host. It's only meant for a few users and not a lot of traffic.
As the ln wallet, we use `phoenixd`.

## Using

**It is strongly encoraged to run this behind a reverse-proxy like nginx**

To use, just build it with rust, and then you'll need a folder with the json of each user you want to support. For example, create a `users/` folder and create one json for each user. The json name should be `user`, where user is what will appear before the `@` on the final address. So `john@smith.com` should have a json called `john` (no `.json`). The content should be the following:

```json
{
	"metadata": "[[\"text/plain\",\"YOUR DESCRIPTION\"]]",
	"callback": "https://<CALLBACK ADDRESS>/callback"
}
```

Replace MIN_SENDABLE and MAX_SENDABLE with something like 1000 and 100000000 (in milisatoshis). `YOUR DESCRIPTION` is a short string that will be shown on the client before paying you. `CALLBACK ADDRESS` is the address where this software is hosted.

Here's an example:

```json
{
	"metadata": "[[\"text/plain\",\"my ln address\"]]",
	"callback": "https://smith.com/callback"
}
```

After that, you need to start `phoenixd` and get the password from `~/.phoenix/phoenix.conf`. You'll see a field like `http-password=<PASSWORD>`. Only copy the `PASSWORD` part. The start this with

```bash
$ ln-address <PASSWORD> --secret-key <a 32-bytes hex-encoded private key for the zap server>
```

Run `--help` to see all options.

### Using with docker

Copy/rename `.env.sample` to `.env` and fill the `PHOENIXD_PASSWORD` environment variable with your local password, then run the usual `docker compose up` command. 

Attention: this intentionally does not include a container for `phoenixd`. 

