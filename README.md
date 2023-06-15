# I'm Lazy

## Discipline is nice, but...

Truthfully, I've enjoyed writing redirects recently. However, anytime I see a repetitive process (i.e., look at the
`404` report, find the broken link, find the new link, write the redirect, test the redirect, commit the redirect, push
the redirect, and deploy the redirect), I want to automate it. Plus, I wanted to play with Rust 🦀.

## What does this do?

This is a simple Rust application that will take a series of arguments, separated by spaces, and use Algolia — with our
public index, API key, and Application ID — to find the first result. It will then format a redirect for you to add to
our `redirects.conf` file in the `hasura/hasura.io` repository.

## How does it work?

First, you'll need to install Rust. Then, clone the repo:

```bash
git clone https://github.com/robertjdominguez/ez-redirects.git
```

The `default` branch is `feat/fetch-algolia`.

Once it's cloned, you can run the following command to build the binary:

```bash
cargo build
```

Then, you can run the following command to run the binary:

```bash
cargo run  <arguments>
```

Each `<argument>` is a relative path from the docs site. Running the program will look like this:

```bash
cargo run /docs/latest/nested/upserts /docs/latest/enterprise/prometheus /docs/latest/queries/arguments
##################################################################
# DOCS Redirects (06/13/2023)
##################################################################

✅ -> /docs/latest/nested/upserts
✅ -> /docs/latest/enterprise/prometheus
✅ -> /docs/latest/queries/arguments
```

After this output, behind the scenes, the program will create a new branch on `hasura/hasura.io`, add the redirects to
the bottom of the DOCS `404` redirects section of `/redirects/redirects.conf`, and open VS Code.

You can then quickly navigate to them by grepping for the date in MM/DD/YYYY format, test them, and commit them.

## What's next?

~~- [x] Copy the bulk set of redirects and comment to the clipboard~~

☝️ This was eliminated in lieu of the option below. 10x Laziness.

- [x] Or, go big and have it `cd` into the `hasura/hasura.io` repo, add the redirects, and let me take care of the rest
      because god forbid I push something that doesn't work 😬.
