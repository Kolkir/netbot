from message import MessageId, RecvMessage, SendMessage


class GetCameraListMsg(RecvMessage):
    def __init__(self):
        super(GetCameraListMsg, self).__init__(MessageId.GET_CAMERA_LIST)


class SendCameraListMsg(SendMessage):
    def __init__(self):
        super(SendCameraListMsg, self).__init__(MessageId.SEND_CAMERA_LIST)
        self.camera_list = []

    def set_camera_list(self, camera_list):
        self.camera_list = camera_list
        list_len = len(self.camera_list)
        self.add_bytes(list_len.to_bytes(1, byteorder='big'))
        for item in self.camera_list:
            self.add_bytes(item.to_bytes(1, byteorder='big'))
