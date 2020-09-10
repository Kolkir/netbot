import os
import cv2

from client import Client
from message import MessageId, HelloMsg
from camera_msg import *
from image_msg import *


def get_msg_obj(msg_id):
    result = {
        MessageId.HELLO: HelloMsg(),
        MessageId.SEND_IMAGE: SendImageMsg(),
        MessageId.CAPTURE_IMAGE: CaptureImageMsg(),
        MessageId.GET_CAMERA_LIST: GetCameraListMsg(),
        MessageId.SEND_CAMERA_LIST: SendCameraListMsg()
    }.get(msg_id)
    if not result:
        print('Unknown msg_id {}'.format(msg_id))
    return result


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
                frame.shape[1], frame.shape[0]))
            return frame
        else:
            print('Failed to capture an image with width={} and height={}'.format(
                frame_width, frame_height))
    else:
        print('Failed to open the camera {}'.format(camera_id))
    return None


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
        MessageId.CAPTURE_IMAGE: process_capture_image,
        MessageId.GET_CAMERA_LIST: process_get_camera_list,
    }.get(msg.id())(msg)
    return result


def main():
    HOST = '127.0.0.1'  # The server's hostname or IP address
    PORT = 8080         # The port used by the server
    client = Client(process_message, get_msg_obj)
    client.run(HOST, PORT)


if __name__ == '__main__':
    main()
