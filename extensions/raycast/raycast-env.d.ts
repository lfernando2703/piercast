/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** API Base URL - Piercast control plane base URL (no trailing slash). */
  "apiBaseUrl": string,
  /** Bearer Token - Authorization bearer token from pairing.json. */
  "token": string
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `search-apps` command */
  export type SearchApps = ExtensionPreferences & {}
  /** Preferences accessible in the `open-app` command */
  export type OpenApp = ExtensionPreferences & {}
  /** Preferences accessible in the `start` command */
  export type Start = ExtensionPreferences & {}
  /** Preferences accessible in the `stop` command */
  export type Stop = ExtensionPreferences & {}
  /** Preferences accessible in the `restart` command */
  export type Restart = ExtensionPreferences & {}
  /** Preferences accessible in the `kill` command */
  export type Kill = ExtensionPreferences & {}
  /** Preferences accessible in the `expose` command */
  export type Expose = ExtensionPreferences & {}
  /** Preferences accessible in the `daemon-status` command */
  export type DaemonStatus = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `search-apps` command */
  export type SearchApps = {}
  /** Arguments passed to the `open-app` command */
  export type OpenApp = {}
  /** Arguments passed to the `start` command */
  export type Start = {}
  /** Arguments passed to the `stop` command */
  export type Stop = {}
  /** Arguments passed to the `restart` command */
  export type Restart = {}
  /** Arguments passed to the `kill` command */
  export type Kill = {}
  /** Arguments passed to the `expose` command */
  export type Expose = {}
  /** Arguments passed to the `daemon-status` command */
  export type DaemonStatus = {}
}

