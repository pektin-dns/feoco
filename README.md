# **FEOCO**

A container for serving **static** web applications, with **client side routing**, from memory. Intended for use behind a reverse proxy that handles routing and TLS termination.

Created to be a perfect match for serving preact/react/vue or similar apps.

# **265KiB container size**

# **~230k HTTP requests/second**

nginx does ~30k on the same machine

# Important Note

This is **not** a fully featured web server for routing proxying or anything other than serving small single page apps (that use client side routing). You should use traefik, actix, nginx or something else for that.

# Easy Header configuration

## Simple variable replacement out of the box

You can use variables with the variable prefix from the config file:

`config.yml`

```yaml
# set prefix
variable-prefix: "$"
headers:
    all:
        someHeader: $MY_VAR # use variable with prefix from above
# ...
```

The variables will be replaced by the same variable name provided to the container environment.

So when the environment is provided like this...

`docker-compose.yml`

```yaml
version: "3.7"
services:
    feoco-app:
        build:
            context: .
            dockerfile: Dockerfile
        ports:
            - "8080:80"
        environment:
            - MY_VAR=hello
```

...the config from above will be rendered like so:

```yaml
variable-prefix: "$"
headers:
    all:
        someHeader: hello
# ...
```

## Set Headers on all requests and/or on the document seperately

```yaml
variable-prefix: "$"
headers:
    # on every resource
    all:
        This-Header-Will-Be-Set: On All Served Resources
    # only on the document
    document:
        This-Header-Will-Only-Be-Set: On the document (index.html)
```

# Getting started by Example

# IMPORTANT Things to know

All files will be read from the container directory: `/public/`
So you will have to put your files there. This cannot be configured.

The config is expected to be in the container at `/config.yml`
Not `.yaml` Not anywhere else.

An empty default config is mounted in the base image.
It looks like this:

```yaml
variable-prefix: "$"
headers:
    # on every resource
    all: {} # empty "rust hashmap"/"js object"
    # only on the document
    document: {}
```

Your config **has to contain the above fields at minimum** else the server will not launch.

### Your Apps Dockerfile

`Dockerfile`

```Dockerfile
# 0. your build stage
FROM node:16.13.0-alpine3.14 as build-stage

# Add additional needed software
RUN apk add git bash sed
WORKDIR /app

# Build deps
COPY package.json yarn.lock ./
RUN yarn

# Build your app
COPY . .
RUN sh scripts/install-modules.sh
RUN yarn build



# 1. execution stage NOTE: You can't do much here as this is a image from scratch
FROM pektin/feoco
COPY --from=build-stage /app/build/ /public
COPY config.yml /config.yml
```

### Compose File

`docker-compose.yml`

```yaml
version: "3.7"
services:
    feoco-app:
        build:
            context: .
            dockerfile: Dockerfile
        ports:
            - "8080:80"
        environment:
            - CSP_CONNECT_SRC=*
```

### Config File

`config.yml`

```yaml
variable-prefix: "$"
headers:
    # on every resource
    all:
        Strict-Transport-Security: max-age=315360000; includeSubdomains; preload
        Cache-Control: public, max-age=31536000

    # only on the document
    document:
        # You can insert line breaks as you want; they will be removed when loading the config
        Content-Security-Policy: >-
            default-src 'none';

            script-src 'self' 'wasm-unsafe-eval' 'wasm-eval';
                        
            style-src 'self' 
            'sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=' 
            'sha256-gixU7LtMo8R4jqjOifcbHB/dd61eJUxZHCC6RXtUKOQ=' 
            'sha256-Dn0vMZLidJplZ4cSlBMg/F5aa7Vol9dBMHzBF4fGEtk=' 
            'sha256-jX63Mkkw8FdoZGmV5pbbuvq3E6LQBUufPYlkJKSN5T4=' 
            'sha256-1Gz2g8CAv9x9EG1JNQpf4aunCZm7ce4CiOAYSHedtk8=' 
            'sha256-wWWgqv2I1eslvJWGxct2TL1YWfkLJFISQBUcrfymfYI=' 
            'sha256-AviY8ukUNt0M5R4KQLfmyNSp65NLzZO6kpngDHGe2f8='; 

            manifest-src 'self';

            connect-src $CSP_CONNECT_SRC; 

            img-src 'self'; 

            font-src 'self'; 

            base-uri 'none'; 

            form-action 'none'; 

            frame-ancestors 'none';
        x-frame-options: DENY
        x-content-type-options: nosniff
        x-permitted-cross-domain-policies: none
        x-download-options: noopen
        x-xss-protection: 1; mode=block
        referrer-policy: no-referrer
        permissions-policy: accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=(), clipboard-read=(), clipboard-write=(), gamepad=(), speaker-selection=(), conversion-measurement=(), focus-without-user-activation=(), hid=(), idle-detection=(), interest-cohort=(), serial=(), sync-script=(), trust-token-redemption=(), window-placement=(), vertical-scroll=()
```
