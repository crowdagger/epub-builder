ChangeLog
==========

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
