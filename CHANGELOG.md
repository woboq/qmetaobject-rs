# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Tutorial on adding Rust wrappers #171.
- QCoreApplication: wrappers around public static getters & setters #185.

### Deprecated
- Rename QObjectDescription in favor of its new name RustQObjectDescriptor #172.

### Removed
- No longer set QT_SELECT environment variable when running qmake #173.

### Fixed
- Build scripts improvements #174, d7c6587.

## 0.2.2 - 2021-06-28

 - Added QVariant conversion from QObjectPinned
 - Added release_resources to QQuickItem
 - Fix non-MSVC Windows build and cross compilation
 - Fixed SimpleListItem when not QVariant or QByteArray are not in scope

## 0.2.1 - 2021-05-22

 - Qt6 support
 - allow to select qt with env variables QT_INCLUDE_PATH and QT_LIBRARY_PATH
 - Added more features to link to more modules
 - Added a prelude
 - Set the rpath on linux

## 0.2.0 - 2021-04-21


