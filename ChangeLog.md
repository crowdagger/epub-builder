ChangeLog
==========
	
0.4.2 (2019-03-19)
========================
* Add `test` method to `ZipCommand` to test whether this zip command is installed
  on the system

0.4.1 (2019-03-07)
========================
* Replace '\' by '/' in zip files path under Windows

0.4.0 (2018-06-01)
==================
* Fix XML format (thanks to @baskerville)
* Updated dependencies to latest major version
* Now requires rustc >= 1.20.0

0.3.0 (2017-09-27)
---------------------
* Breaking change: now requires rustc >= 1.14.0.
* Escape quotes in titles when writing content.opf.
* Update chrono dependency to 0.4.
* Update error-chain dependency to 0.11.0.


0.2.3 (2017-06-03)
---------------------
* Update uuid dependency to latest version.

0.2.2 (2017-03-20)
----------------------
* Avoid rendering empty anchors within nav element.

0.2.1 (2017-03-04)
----------------------
* Update dependencies (chrono, uuid, error-chain) to latest versions.

0.2.0 (2017-01-17)
----------------------
Fix the way `mimetype` is stored in the EPUB file (insure it isn't deflated).

This caused some minor breaking changes:
* The `Zip` implementations now take care of adding the `mimetype` file, not
  `EpubBuilder`.
* `ZipLibrary::new()` now returns a `Result<ZipLibrary>`.

Normally, replacing `ZipLibrary::new()` with e.g. `ZipLibrary::new()?` should be
enough to switch to this new version. 
  
0.1.2 (2017-01-13)
----------------------
* Fix the cover image properties in EPUB3.
* Some code cleaning.

0.1.1 (2017-01-04)
----------------------
* Add `add_cover_image` method to `EpubBuilder`.
* Add `ReferenceType` enum and `reftype` method to `EpubContent` to add items to
	the guide section (EPUB2) or the landmarks section of nav.xhtml (EPUB3).

0.1.0 (2017-01-02)
----------------------
* initial release
