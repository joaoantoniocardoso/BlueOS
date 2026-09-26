# blueos_msgs/msg/EndpointInfo
# One key a service serves, listed in ServiceInfo.endpoints so clients can discover the API (D-12, D-24).

# One of: command, query, io_query, state, event.
string kind
string name
string key
# Schema of the payload the client sends (command, query, io_query). Empty when the endpoint takes none.
string request_schema
# Schema of the reply (command: CommandAck; query, io_query) or of the published sample (state, event).
string response_schema
