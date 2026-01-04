# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Fix state transition bug where overheat skips from overheat to coaling, and therefore doesn't trigger the state change 
  to coaling, on which the reload notification relies.
