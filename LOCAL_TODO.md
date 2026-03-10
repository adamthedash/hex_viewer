# Idea for binary format editor

**features**
- Show parser applied to many files in parallel - 1 line per file
- Show binary data (hex/byte-aligned decimal), toggle to show structs
- Highlight higherarchy of parsers with colours, with corresponding highlights over the binary
- Horizontally align binary data across different files
- Hover over parser to highlight corresponding section in binary & vice versa
- Lazy eval of parser - eg. if you know the start/end but not what's in the middle (regex lookahead might be similar?)
- Interactive data type hover tool - Select f32le, hover over any part of the binary to see what it would be like to interpret that section as f32le. Or maybe just one "interpret" tool that shows many different data types when hovering on a section.
- On failure, show stuff correctly parsed before it, the section of binary where the fail occurred, and the corresponding parser which failed.

**impl**
- Egui or ratatui for UI. Or maybe Dioxus if using subsecond?
- Winnow as backing parsers. Will need to explore span tracking.  
- Subsecond for hot-reloading (reloading structs can be a bit weird apparently)  
- Parsers defined externally using a normal editor rather than trying to create an in-app editor  

