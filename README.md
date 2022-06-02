## Overview

Static site generator.

> :warning: **WORK IN PROGRESS**

## Setup

The folder `.docrs/` contains the configuration: config file, templates, etc...

- `.docsrs/config.toml` contains the template to use, the pages to render, and other settings.
- `.docrs/templates` containts the templates/themes.

### Templates

A template defines the following files:

- `index.hbs`: Template for the homepage.
- `page.hbs`: Template for individual pages.
- static files (js utilitiies, css styles, static assets, etc...)

NB: If an page named `index` is defined at the root, it will overwrite the default index page.

The `.hbs` templates are Handlebars templates, and are fed the following variables:

- `content`: HTML content of each page

In order to define a custom template, simply duplicate and adjust a theme folder within the `.docrs/templates` folder. The folder name is used as the template name.

## API

### `docrs init`

Initializes the repo with a `.docrs/` configuration folder.

### `docrs dev`

Builds the site, with watch mode and live reload.

### `docrs build`

Builds the site.

### `docrs serve`

Serves the build folder.
