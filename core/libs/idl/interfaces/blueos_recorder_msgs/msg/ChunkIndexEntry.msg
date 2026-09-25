# blueos_recorder_msgs/msg/ChunkIndexEntry

uint64 start_time
uint64 end_time
# Offset and length of the whole Chunk record, header included.
uint64 offset
uint64 length
string compression
uint64 compressed_size
uint64 uncompressed_size
uint16[] channel_ids
# Bytes of the MessageIndex records that follow the chunk.
uint64 message_index_length
