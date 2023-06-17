# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## 0.2.9 2023-06-15

 - Implement QMetaType for QStringList and QVariantMap
 - Added QGlobalColor
 - implemented Add and AddAsing for QString
 - Added QVariant::is_null
 - Expose begin_move_rows and end_move_rows
 - Fixed UTF8 in QJSonObject
 - Fixed compulation with Qt 6.5

## 0.2.8 2022-02-25 (qttype only)

 - Fix qttype crate always being rebuilt (#252)

## 0.2.7 2022-02-23

 - Expand the API of QStringList, QString
 - Expand the API of QJSValue
 - Added QVariantMap
 - Add wrapper around qmlRegisterModule
 - Fix compilation with Qt 6.3 and MSVC
 - qttypes: Expose the flags through a COMPILE_FLAGS env variable


## 0.2.6 2021-11-19 (qttype only)

 - Fix build when Qt is not found and the required feature is off

## 0.2.5 2021-11-19

 - Completed QColor API
 - Added wrapper around QJSon* types, QPainter, QPen, QBrush, QLineF
 - Added QQuickPaintedItem
 - Fixes to the qttype build script

## 0.2.4 2021-09-30

- Fixed build with Qt < 5.8 and >= 6.2

## 0.2.3 2021-08-11

### Added
- Tutorial on adding Rust wrappers #171.
- QCoreApplication: wrappers around public static getters & setters #185.

### Deprecated
- Rename QObjectDescription in favor of its new name RustQObjectDescriptor #172.

### Removed
- No longer set QT_SELECT environment variable when running qmake #173.

### Fixed
- Build scripts improvements #174, d7c6587.
- Symbol required for the QEnum macro are in the prelude

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


