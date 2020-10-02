import os
import cv2
import argparse

from client import Client
from message import MessageId, HelloMsg
from camera_msg import *
from image_msg import *
from move_msg import *
from camera_prop_msg import *


def get_msg_obj(msg_id):
    result = {
        MessageId.HELLO: HelloMsg(),
        MessageId.SEND_IMAGE: SendImageMsg(),
        MessageId.CAPTURE_IMAGE: CaptureImageMsg(),
        MessageId.GET_CAMERA_LIST: GetCameraListMsg(),
        MessageId.SEND_CAMERA_LIST: SendCameraListMsg(),
        MessageId.MOVE: MoveMsg(),
        MessageId.GET_CAMERA_PROP: GetCameraPropMsg(),
        MessageId.SEND_CAMERA_PROP: SendCameraPropMsg(),
    }.get(msg_id)
    if not result:
        print('Unknown msg_id {}'.format(msg_id))
    return result


def get_camera_indices():
    # checks the first 10 indexes.

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


def get_camera_prop(camera_id):
    resolutions = [(320, 240), (640, 480), (800, 600), (1024, 768),
                   (960, 680), (1280, 720), (1440, 720), (1920, 1080)]
    cap = cv2.VideoCapture(camera_id, cv2.CAP_V4L)
    if cap.isOpened():
        arr = []
        for resolution in resolutions:
            cap.set(cv2.CAP_PROP_FRAME_WIDTH, resolution[0])
            cap.set(cv2.CAP_PROP_FRAME_HEIGHT, resolution[1])
            is_captured, frame = cap.read()
            if is_captured and frame.shape[1] == resolution[0] and frame.shape[0] == resolution[1]:
                arr.append(resolution[0])
                arr.append(resolution[1])
        cap.release()
        return arr
    else:
        print('Failed to open the camera {}'.format(camera_id))
    return None


def capture_image(camera_id, frame_width, frame_height):
    cam = cv2.VideoCapture(camera_id, cv2.CAP_V4L)
    if cam.isOpened():
        cam.set(cv2.CAP_PROP_FRAME_WIDTH, frame_width)
        cam.set(cv2.CAP_PROP_FRAME_HEIGHT, frame_height)
        cam.set(cv2.CAP_PROP_BACKLIGHT, 0)
        is_captured, frame = cam.read()
        if is_captured:
            print("Frame was captured: width {} height {}".format(
                frame.shape[1], frame.shape[0]))
            frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            return frame_rgb
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


def process_camera_prop(msg):
    print("Getting camera prop")
    camera_props = get_camera_prop(msg.camera_id)
    response = SendCameraPropMsg()
    response.set_camera_prop(camera_props)
    return response


def process_move(msg):
    print("Moving left {}:{} right {}:{}".format(
        msg.left_speed, msg.left_dir, msg.right_speed, msg.right_dir))


def process_message(msg):
    # print("Processing msg id : {}".format(msg.id()))
    result = {
        MessageId.CAPTURE_IMAGE: process_capture_image,
        MessageId.GET_CAMERA_LIST: process_get_camera_list,
        MessageId.MOVE: process_move,
        MessageId.GET_CAMERA_PROP: process_camera_prop,
    }.get(msg.id())(msg)
    return result


def main():
    parser = argparse.ArgumentParser(description='Starts NetBot client.')
    parser.add_argument('host', action="store")
    parser.add_argument('--port', action="store", dest="port", default=2345,
                        type=int, required=False)
    args = parser.parse_args()
    host = args.host
    port = args.port
    if (host == None) | (port == None):
        print(parser.usage)
        exit(0)
    else:
        client = Client(process_message, get_msg_obj)
        client.run(host, port)


if __name__ == '__main__':
    main()
