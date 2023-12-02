# Unfurl
_Unfurl_ expands links in text and replaces them with useful content.

```
$ unfurl <<EOF
This issue was resolved: https://github.com/bww/unfurl/issues/1
EOF
This issue was resolved: This is just an example issue (#1)
$
```

## Supported services and routes
Unfurl supports expanding the following URL types out of the box:

* **Github**
    * PRs
      `https://api.github.com/repos/{org}/{repo}/pulls/{num}'`
    * Issues
      `https://api.github.com/repos/{org}/{repo}/issues/{num}`

* **Jira Cloud**
    * Issues
      `https://{domain}/rest/api/3/issue/{key}`

You can add support for more services by configuring a routes definition and specifying it on the command line via `--routes <definition.yml>`. The [built-in routes definition file](https://github.com/bww/unfurl/blob/master/conf/routes.yml) can be used as reference. Custom routes are appended to the built-in routes and take prescidence.

## Authenticating to services
Out of the box, Unfurl will work as expected for supported public URLs. Often, however, URLs hosted on these services are not public, so you may need to provide some credentials. This can be done via a configuration file, located by default at `$HOME/.unfurl.yml`.

## Custom output formats
It is also possible to specify how, exactly, URLs are expanded by defining a per-URL formatting template. The input to this format is the JSON received from the underlying service, so you can use any information that is made available through the service's APIs.

## Example configuration file

```yaml
 services:

    # Provide authentication and custom formatting for GitHub routes. You
    # cannot define routes in a configuration file, use a route definition
    # file for that.
    github.com:
      auth: # optionally provide authentication to expand non-public URLs
        header: Bearer $YOUR_PERSONAL_ACCESS_TOKEN
      format: # optionally define formats for these URL patterns
        pr: "[{number}] {title} ({url})"
        issue: "[{number}] {title} ({url})"

    # Jira configurations are per-cloud-tenant; specify your domain. (Unfurl
    # will fail over to the base domain, `atlassian.net` in this case, if no
    # exact match is found.)
    treno.atlassian.net: 
      auth:
        header: Basic $YOUR_PERSONAL_ACCESS_CREDENTIALS
      format:
        issue: "[{key}] {fields.summary}"
    
    # And so on for whatever other domains you have added support for.
    twitter.com:
      auth:
        header: # ...  

```
