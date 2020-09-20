
from message import MessageId, RecvMessage


class MoveMsg(RecvMessage):
    def __init__(self):
        super(MoveMsg, self).__init__(MessageId.MOVE)
        self.left_speed = 0
        self.left_dir = 0
        self.right_speed = 0
        self.right_dir = 0

    def from_bytes(self, data):
        self.left_speed = int.from_bytes(data[0:1], byteorder='big')
        self.left_dir = int.from_bytes(data[1:2], byteorder='big')
        self.right_speed = int.from_bytes(data[2:3], byteorder='big')
        self.right_dir = int.from_bytes(data[3:4], byteorder='big')
