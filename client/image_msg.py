from message import MessageId, RecvMessage, SendMessage


class CaptureImageMsg(RecvMessage):
    def __init__(self):
        super(CaptureImageMsg, self).__init__(MessageId.CAPTURE_IMAGE)
        self.camera_id = 0
        self.img_width = 0
        self.img_height = 0

    def from_bytes(self, data):
        self.camera_id = int.from_bytes(data[0:1], byteorder='big')
        self.img_width = int.from_bytes(data[1:3], byteorder='big')
        self.img_height = int.from_bytes(data[3:5], byteorder='big')


class SendImageMsg(SendMessage):
    def __init__(self):
        super(SendImageMsg, self).__init__(MessageId.SEND_IMAGE)
        self.img_width = 0
        self.img_height = 0
        self.img_channels = 0

    def set_img(self, data, channels, width, height):
        size = 2 + 2 + 2 + (width * height * channels)
        self.size_ = 1 + size
        self.img_width = width
        self.img_height = height
        self.img_channels = channels
        self.add_bytes(self.img_channels.to_bytes(2, byteorder="big"))
        self.add_bytes(self.img_width.to_bytes(2, byteorder="big"))
        self.add_bytes(self.img_height.to_bytes(2, byteorder="big"))
        self.add_bytes(data)
