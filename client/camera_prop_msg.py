
from message import MessageId, RecvMessage, SendMessage


class GetCameraPropMsg(RecvMessage):
    def __init__(self):
        super(GetCameraPropMsg, self).__init__(MessageId.GET_CAMERA_PROP)
        self.camera_id = 0

    def from_bytes(self, data):
        self.camera_id = int.from_bytes(data[0:1], byteorder='big')


class SendCameraPropMsg(SendMessage):
    def __init__(self):
        super(SendCameraPropMsg, self).__init__(MessageId.SEND_CAMERA_PROP)
        self.prop_list = []
        self.camera_id = 0

    def set_camera_prop(self, prop_list):
        self.prop_list = prop_list
        self.add_bytes(self.camera_id.to_bytes(1, byteorder="big"))
        list_len = len(self.prop_list)
        self.add_bytes(list_len.to_bytes(2, byteorder='big'))
        for item in self.prop_list:
            self.add_bytes(item.to_bytes(2, byteorder='big'))


class SetCameraPropMsg(RecvMessage):
    def __init__(self):
        super(SetCameraPropMsg, self).__init__(MessageId.SET_CAMERA_PROP)
        self.camera_id = 0
        self.frame_width = 0
        self.frame_height = 0
        self.fps = 0
        self.do_encoding = 0

    def from_bytes(self, data):
        self.camera_id = int.from_bytes(data[0:1], byteorder='big')
        self.frame_width = int.from_bytes(data[1:3], byteorder='big')
        self.frame_height = int.from_bytes(data[3:5], byteorder='big')
        self.fps = int.from_bytes(data[5:6], byteorder='big')
        self.do_encoding = int.from_bytes(data[6:7], byteorder='big')
