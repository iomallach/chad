## Random ideas
- Number of connected clients [DONE]
- Status bar "component" with evenly spaced writings [DONE][Requires coord refactoring][DONE]
- Header "component"
- Prompt is on the last line [DONE]
- Allow capitalized letters [DONE]
- Fix Connection reset by peer after read_message
- Decompose Message into Headers and embed Headers into Message. Rename message field to be body instead
- Put chat into a frame (draw indents around using some background color)

## Features
- Authorization
- Broadcasting "entered chat" message [DONE]
- "Left the chat" message
- Proper parser for messages [DONE]
- Show online users somewhere
- In-message markdown support
- Put all constants into configuration (bar placements, max messages, etc)

### Failures
- Clients panic when a client disconnects on unwrap somewhere
- There is a bug when previous messages concat with new messages after popping from the queue -- supposedly, need to confirm
- There are lots of warnings

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
