import os
import cv2
import argparse

from client import Client
from message import MessageId, HelloMsg, StopMsg
from camera_msg import *
from image_msg import *
from move_msg import *
from camera_prop_msg import *

cameras = dict()


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
        MessageId.STOP: StopMsg(),
        MessageId.SET_CAMERA_PROP: SetCameraPropMsg()
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
            print('Camera {} detected\n'.format(index))
            frame = cap.read()
            if (frame[0]):
                print('is working\n')
                arr.append(index)
                cameras[index] = cap
        index += 1
    return arr


def get_camera_prop(camera_id):
    resolutions = [(320, 240), (640, 480), (800, 600), (1024, 768),
                   (960, 680), (1280, 720), (1440, 720), (1920, 1080)]
    cap = cameras[camera_id]
    if cap.isOpened():
        arr = []
        for resolution in resolutions:
            cap.set(cv2.CAP_PROP_FRAME_WIDTH, resolution[0])
            cap.set(cv2.CAP_PROP_FRAME_HEIGHT, resolution[1])
            cap.set(cv2.CAP_PROP_FPS, 30)
            is_captured, frame = cap.read()
            if is_captured and frame.shape[1] == resolution[0] and frame.shape[0] == resolution[1]:
                arr.append(resolution[0])
                arr.append(resolution[1])
        return arr
    else:
        print('Failed to open the camera {}'.format(camera_id))
    return None


def set_camera_prop(camera_id, frame_width, frame_height):
    cam = cameras[camera_id]
    if cam.isOpened():
        cam.set(cv2.CAP_PROP_FRAME_WIDTH, frame_width)
        cam.set(cv2.CAP_PROP_FRAME_HEIGHT, frame_height)
        cam.set(cv2.CAP_PROP_BACKLIGHT, 0)
        cam.set(cv2.CAP_PROP_FPS, 30)
        is_captured, frame = cam.read()
        if not is_captured:
            print('Failed to set camera prop with width={} and height={}'.format(
                frame_width, frame_height))
    else:
        print('Failed to open the camera {}'.format(camera_id))
    return None


def capture_image(camera_id):
    cam = cameras[camera_id]
    if cam.isOpened():
        is_captured, frame = cam.read()
        if is_captured:
            # print("Frame was captured: width {} height {}".format(
            #    frame.shape[1], frame.shape[0]))
            frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            is_encoded, buffer = cv2.imencode('.png', frame_rgb)
            if not is_encoded:
                print('Failed to encode captured frame')
                return None
            return buffer, frame_rgb.shape
        else:
            print('Failed to capture an image')
    else:
        print('Failed to open the camera {}'.format(camera_id))
    return None


def process_set_camera_prop(msg):
    print("Set camera props: camera {} width {} height {}".format(
        msg.camera_id, msg.frame_width, msg.frame_height))
    set_camera_prop(msg.camera_id, msg.frame_width, msg.frame_height)


def process_capture_image(msg):
    # print("Capturing image: camera {}")
    img, shape = capture_image(msg.camera_id)
    response = SendImageMsg()
    response.set_img(msg.camera_id, img, shape[2], shape[1], shape[0])
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


def process_stop(msg):
    print('Stopping...')
    for _, cam in cameras.items():
        cam.release()
    return StopMsg()


def process_message(msg):
    # print("Processing msg id : {}".format(msg.id()))
    result = {
        MessageId.CAPTURE_IMAGE: process_capture_image,
        MessageId.GET_CAMERA_LIST: process_get_camera_list,
        MessageId.MOVE: process_move,
        MessageId.GET_CAMERA_PROP: process_camera_prop,
        MessageId.STOP: process_stop,
        MessageId.SET_CAMERA_PROP: process_set_camera_prop,
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
