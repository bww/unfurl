# Unfurl
_Unfurl_ expands links in text and replaces them with useful content.

```
$ unfurl <<EOF
This issue was resolved: https://github.com/bww/unfurl/issues/1
EOF
This issue was resolved: This is just an example issue (#1)
$
```

## Configuring Unfurl
Unfurl currently supports expanding the following URLs:

* **Github**
    * PRs
    * Issues
* **Jira Cloud**
    * Issues

Out of the box, Unfurl will work as expected for supported public URLs. Often, however, URLs hosted on these services are not public, so you may need to provide some credentials. This can be done via a configuration file, located by default at `$HOME/.unfurl.yml`.

It is also possible to specify how, exactly, URLs are expanded by defining a per-URL formatting template. The input to this format is the JSON received from the underlying service, so you can use any information that is made available through the service's APIs.

```yaml
 services:

    github.com:
      auth: # optionally provide authentication to expand non-public URLs
        header: Bearer $YOUR_PERSONAL_ACCESS_TOKEN
      format: # optionally define formats for these URL patterns
        pr: "[{number}] {title} ({url})"
        issue: "[{number}] {title} ({url})"

    treno.atlassian.net: # Jira configurations are per-cloud-tenant; specify your domain
      auth:
        header: Basic $YOUR_PERSONAL_ACCESS_CREDENTIALS
      format:
        issue: "[{key}] {fields.summary}"

```
