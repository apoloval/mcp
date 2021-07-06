* v0.3.0
  - Fix several issues in modern Rust
  - Upgrade all dependencies

* v0.2.0
  - Deep refactor of file management operations.
  - Bug fixing
     - While extracting, a CAS package containing files with repeated names causes the last file to overwrite the first. Now in case of name clashing, the subsequent file is extracted with name `<filename>-1.<ext>`. If this is already occupied, `<filename>-2.<ext>` is used, and so on.
     - While extracting, a CAS package containing blank filenames cause they to be extracting without file stem. For instance, a blank binary file is extracted as `.bin`. This is a hidden file in UNIXes. This is fixed by replacing blank names by `noname`.

* v0.1.0
  - Initial version

