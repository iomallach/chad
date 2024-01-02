## Random ideas
- Number of connected clients [DONE]
- Status bar "component" with evenly spaced writings [DONE][Requires coord refactoring]
- Header "component"
- Prompt is on the last line [DONE]
- Allow capitalized letters [DONE]
- Fix Connection reset by peer after read_message
- Decompose Message into Headers and embed Headers into Message. Rename message field to be body instead

## Features
- Authorization
- Broadcasting "entered chat" message
- Proper parser for messages
- Show online users somewhere
- In-message markdown support

### Failures
- Clients panic when a client disconnects on unwrap somewhere

### Parser idea
"header1: value1 \r\n header2: value2 \r\n\r\n some message \r\n\r\n"

## Algo
peek next element
if is_alphanumeric()
consume until \r\n
match header or generate error
if next is empty line consume the body using content-length
else repeat peek next element (continue main loop)
-------------------
read line
if empty line and content-length != 0 consume content-length bytes and then terminate
else match header
