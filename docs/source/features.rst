Features
========

Asynchronous Operations
-----------------------

`lazyflat` uses `tokio` to perform all Flatpak operations (search, install, update, uninstall) in the background. This ensures that the user interface remains responsive and fluid even during long-running tasks.

Tabs
----

- **Up to Date**: Shows all installed Flatpak applications that are currently at their latest version.
- **Updates**: Lists applications with available updates. You can update them individually or all at once.
- **Runtimes**: Displays installed Flatpak runtimes (libraries and platforms).
- **Discover**: Allows searching for new applications from configured remotes and installing them.

Search
------

You can search within any tab by pressing `/`. In the **Discover** tab, pressing `Enter` after typing a query will trigger a remote search.

Mouse Interaction
-----------------

The UI supports mouse scrolling for navigation and left-clicking to switch tabs or select items in the list.
