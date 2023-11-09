# Unfurl
_Unfurl_ expands links in text and replaces them with useful content.

```
$ unfurl <<EOF
The following issues were resolved:

  * https://github.com/bww/unfurl/issues/1
  * https://github.com/bww/unfurl/issues/2
  * https://github.com/bww/unfurl/issues/3

EOF

The following issues were resolved:

  * A (#1)
  * B (#1)
  * C (#1)

$
```
