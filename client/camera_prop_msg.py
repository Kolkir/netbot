
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

    def set_camera_prop(self, camera_list):
        self.prop_list = prop_list
        list_len = len(self.prop_list)
        self.add_bytes(list_len.to_bytes(2, byteorder='big'))
        for item in self.camera_list:
            self.add_bytes(item.to_bytes(2, byteorder='big'))
