from enum import IntEnum


class MessageId(IntEnum):
    HELLO = 1
    CAPTURE_IMAGE = 2
    SEND_IMAGE = 3
    GET_CAMERA_LIST = 4
    SEND_CAMERA_LIST = 5
    MOVE = 6


class Message:
    def __init__(self, id):
        self.id_ = id

    def id(self):
        return self.id_


class RecvMessage(Message):
    def __init__(self, id):
        super(RecvMessage, self).__init__(id)

    def from_bytes(self, data):
        pass


class SendMessage(Message):
    def __init__(self, id):
        super(SendMessage, self).__init__(id)
        self.bytes_ = bytearray()

    def size(self):
        return len(self.bytes_)

    def to_bytes(self):
        return self.bytes_

    def add_bytes(self, data):
        self.bytes_.extend(data)


class HelloMsg(Message):
    def __init__(self):
        super().__init__(MessageId.HELLO)

    def size(self):
        return 0

    def to_bytes(self):
        return None

    def from_bytes(self, data):
        pass
