# CFPROXY - The curseforge proxy server

Curseforge has locked down their API and now restricts access without authentification. This spells trouble for application developers using the API  - They aren't allowed to ship their API Key to their users, and must resort to alternate methods. This is one of them.

## What does it do?

> Runs a HTTP server. Every request made to it gets proxied to the Curseforge API with an API key attached - that way, application devs can use this as the endpoint for their CF requests, and the CF API key stays safe, only being used on the server.

There is no authentification, but to prevent API abuse, requests get rate limited. (If you want to run a private proxy server, you can turn this off.)

## Docs

Every request can be made as seen in [the Curseforge API docs](https://docs.curseforge.com/#getting-started), but instead of using `https://api.curseforge.com` as the base url, use *whatever url this server runs at*.

Both headers, path, params and body should be proxied to CF, if you notice something odd or think something doesn't get proxied, please open an issue.

## Building from source

If you want to run the server yourself! Everything you need is [Rust](https://www.rust-lang.org/) installed and your Curseforge API Key (if you don't have one, apply [here](https://forms.monday.com/forms/dce5ccb7afda9a1c21dab1a1aa1d84eb?r=use1) - this will take a while).

- Clone the repository.
- Put your API key into an environment variable named `CF_API_KEY` (You can also use an `.env` file with `CF_API_KEY = '..'`. Don't forget the single quotes!)
- Run the server with `cargo run`
- You should see a message popping up: `Server starting at port 3000`. Success! You can now make requests to your server.

Additional options are configured through environment variables:

| Key | Value type | Meaning |
| --- | ---------- | ------- |
| `CF_API_KEY` | string | Your API key you got from Curseforge.
| `PORT` | number | The port at which to start up the server. Optional - defaults to `3000`.
| `REQ_LIMIT_PER_SEC` | number | How many requests per second per IP address are allowed. Optional - defaults to `6`.