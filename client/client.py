import os
import cv2
import socket
from enum import IntEnum


def get_camera_indices():
    # checks the first 10 indexes.

    # fps = 10
    index = 0
    arr = []
    for _ in range(10):
        print('Trying camera with index {}\n'.format(index))
        cap = cv2.VideoCapture(index, cv2.CAP_V4L)
        if cap.isOpened():
            # cap.set(cv2.CAP_PROP_FRAME_WIDTH, frame_width)
            # cap.set(cv2.CAP_PROP_FRAME_HEIGHT, frame_height)
            # cap.set(cv2.CAP_PROP_FPS, fps)
            print('Camera {} detected\n'.format(index))
            frame = cap.read()
            if (frame[0]):
                print('is working\n')
                arr.append(index)
            cap.release()
        index += 1
    return arr


def capture_image(camera_id, frame_width, frame_height):
    cam = cv2.VideoCapture(camera_id, cv2.CAP_V4L)
    if cam.isOpened():
        cam.set(cv2.CAP_PROP_FRAME_WIDTH, frame_width)
        cam.set(cv2.CAP_PROP_FRAME_HEIGHT, frame_height)
        is_captured, frame = cam.read()
        if is_captured:
            print("Frame was captured: width {} height {}".format(
                frame.shape[1], frame.shape[2]))
            return frame
        else:
            print('Failed to capture an image with width={} and height={}'.format(
                frame_width, frame_height))
    else:
        print('Failed to open the camera {}'.format(camera_id))
    return None


class MsgId(IntEnum):
    HELLO = 1
    CAPTURE_IMAGE = 2
    SEND_IMAGE = 3
    GET_CAMERA_LIST = 4
    SEND_CAMERA_LIST = 5


class Message:
    def __init__(self, id):
        self.id_ = id
        self.size_ = 1
        self.bytes_ = bytearray(self.id_.to_bytes(self.size_, "big"))

    def id(self):
        return self.id_

    def size(self):
        return self.size_

    def to_bytes(self):
        return self.bytes_

    def add_bytes(self, data):
        self.bytes_.extend(data)

    def read_data(self, data):
        if len(data) > 0:
            self.id_ = int.from_bytes(data[0:1], byteorder='big')


class GetCameraListMsg(Message):
    def __init__(self):
        super().__init__(MsgId.GET_CAMERA_LIST)


class SendCameraListMsg(Message):
    def __init__(self):
        super().__init__(MsgId.SEND_CAMERA_LIST)
        self.camera_list = []

    def set_camera_list(self, camera_list):
        self.camera_list = camera_list

    def size(self):
        return len(self.bytes_)

    def to_bytes(self):
        list_len = len(self.camera_list)
        self.add_bytes(list_len.to_bytes(1, byteorder='big'))
        for item in self.camera_list:
            self.add_bytes(item.to_bytes(1, byteorder='big'))
        return super().to_bytes()


class CaptureImageMsg(Message):
    def __init__(self):
        super().__init__(MsgId.CAPTURE_IMAGE)
        self.camera_id = 0
        self.img_width = 0
        self.img_height = 0
        self.size_ = 1 + 1 + 2 + 2

    def read_data(self, data):
        self.camera_id = int.from_bytes(data[0:1], byteorder='big')
        self.img_width = int.from_bytes(data[1:3], byteorder='big')
        self.img_height = int.from_bytes(data[3:5], byteorder='big')


class SendImageMsg(Message):
    def __init__(self):
        super().__init__(MsgId.SEND_IMAGE)
        self.img_width = 0
        self.img_height = 0
        self.img_channels = 0

    def set_img(self, data, channels, width, height):
        size = 2 + 2 + 2 + (width * height * channels)
        self.size_ = 1 + size
        self.img_width = width
        self.img_height = height
        self.img_channels = channels
        self.add_bytes(size.to_bytes(4, byteorder="big"))
        self.add_bytes(self.img_channels.to_bytes(2, byteorder="big"))
        self.add_bytes(self.img_width.to_bytes(2, byteorder="big"))
        self.add_bytes(self.img_height.to_bytes(2, byteorder="big"))
        self.add_bytes(data)


class HelloMsg(Message):
    def __init__(self):
        super().__init__(MsgId.HELLO)


def get_msg_obj(msg_id):
    result = {
        MsgId.HELLO: HelloMsg(),
        MsgId.SEND_IMAGE: SendImageMsg(),
        MsgId.CAPTURE_IMAGE: CaptureImageMsg(),
        MsgId.GET_CAMERA_LIST: GetCameraListMsg(),
        MsgId.SEND_CAMERA_LIST: SendCameraListMsg()
    }.get(msg_id)
    if not result:
        print('Unknown msg_id {}'.format(msg_id))
    return result


def process_capture_image(msg):
    print("Capturing image: camera {} width {} height {}".format(
        msg.camera_id, msg.img_width, msg.img_height))
    img = capture_image(msg.camera_id, msg.img_width, msg.img_height)
    response = SendImageMsg()
    response.set_img(img.data, img.shape[2], img.shape[1], img.shape[0])
    return response


def process_get_camera_list(msg):
    print("Getting camera list")
    camera_list = get_camera_indices()
    response = SendCameraListMsg()
    response.set_camera_list(camera_list)
    return response


def process_message(msg):
    print("Processing msg id : {}".format(msg.id()))
    result = {
        MsgId.CAPTURE_IMAGE: process_capture_image,
        MsgId.GET_CAMERA_LIST: process_get_camera_list,
    }.get(msg.id())(msg)
    return result


def run_socket_client(host, port):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        # handshakes
        hello_msg = HelloMsg()
        s.sendall(hello_msg.to_bytes())
        data = s.recv(hello_msg.size())
        hello_msg.read_data(data)
        if MsgId.HELLO == hello_msg.id():
            print('Handshake completed')
            done = False
            while not done:
                msg_id = s.recv(1)
                if len(msg_id) == 0:
                    done = True
                else:
                    msg_id = int.from_bytes(msg_id, byteorder='big')
                    print("Got msg_id {}".format(msg_id))
                    msg = get_msg_obj(msg_id)
                    data = s.recv(msg.size() - 1)
                    msg.read_data(data)
                    response = process_message(msg)
                    s.sendall(response.to_bytes())
            print('Connection closed')
        else:
            print('Handshake failed, msg_id={}'.format(msg_id))
            exit(1)


def main():
    HOST = '127.0.0.1'  # The server's hostname or IP address
    PORT = 8080         # The port used by the server

    run_socket_client(HOST, PORT)


if __name__ == '__main__':
    main()
