Github Rate Limit Monitor
=========================

`grlm` is a small and funny commandline tool written in rust.
It polls the github api [rateLimit](https://developer.github.com/v3/rate_limit/) endpoint and fills a nice little progressbar.

![screencast](docs/grlm.gif)

Usage
-----

```bash
grlm - github rate limit monitor

Usage:
  grlm [(-l <user> -p <password> | -t <token>)] [options]
  grlm --version
  grlm -h | --help

Options:
  -l <user>, --login <user>                the github username
  -p <password>, --password <password>     the user password
  -t <token>, --access-token <token>       an github accesstoken
  -f <frequency>, --frequency <frequency>  refresh freqency [default: 10]
  -r <resource>, --resource <resource>     define which github resource to show
                                           Valid values: core, search, graphql [default: core]
  -V, --version                            print version
  -h, --help                               show this help message and exit
```

Installation
------------

### From Homebrew

```
brew tap wooga/tools
brew install grlm
```

### From Source

1. Git clone the repo and `cd` into the directory. 
2. marke install

License
-------

[MIT License](http://opensource.org/licenses/MIT).

