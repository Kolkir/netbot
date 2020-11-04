import os
import cv2
import argparse
import threading
import time

from client import Client
from message import MessageId, HelloMsg, StopMsg
from camera_msg import *
from image_msg import *
from move_msg import *
from camera_prop_msg import *

from chassis import Chassis

cameras = dict()
cameras_dict_lock = threading.RLock()
cameras_locks = dict()
cameras_encoding = dict()
fps = 30
stop_event = threading.Event()


def image_capture_thread_func(client):
    done = False
    while not done:
        with cameras_dict_lock:
            cam_ids = cameras.keys()

        for cam_id in cam_ids:
            img, shape, encoded = capture_image(cam_id)
            response = SendImageMsg()
            response.set_img(
                cam_id, img, shape[2], shape[1], shape[0],  encoded)
            client.send_msg(response)
            time.sleep(1.0/fps)
        done = stop_event.is_set()


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
                with cameras_dict_lock:
                    cameras[index] = cap
                    cameras_locks[index] = threading.RLock()
                    cameras_encoding[index] = False
        index += 1
    return arr


def get_camera_prop(camera_id):
    lock = cameras_locks[camera_id]
    with lock:
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


def set_camera_prop(camera_id, frame_width, frame_height, fps, do_encoding):
    lock = cameras_locks[camera_id]
    with lock:
        cam = cameras[camera_id]
        if cam.isOpened():
            if frame_width != 0:
                cam.set(cv2.CAP_PROP_FRAME_WIDTH, frame_width)
            if frame_height != 0:
                cam.set(cv2.CAP_PROP_FRAME_HEIGHT, frame_height)
            cam.set(cv2.CAP_PROP_BACKLIGHT, 0)
            if fps != 0:
                cam.set(cv2.CAP_PROP_FPS, fps)
            cameras_encoding[camera_id] = do_encoding
            is_captured, _ = cam.read()
            if not is_captured:
                print('Failed to set camera prop with width={} and height={}'.format(
                    frame_width, frame_height))
        else:
            print('Failed to open the camera {}'.format(camera_id))
        return None


def capture_image(camera_id):
    lock = cameras_locks[camera_id]
    with lock:
        cam = cameras[camera_id]
        if cam.isOpened():
            is_captured, frame = cam.read()
            if is_captured:
                # print("Frame was captured: width {} height {}".format(
                #    frame.shape[1], frame.shape[0]))
                frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                do_encoding = cameras_encoding[camera_id]
                if do_encoding:
                    is_encoded, buffer = cv2.imencode('.png', frame_rgb)
                    if not is_encoded:
                        print('Failed to encode captured frame')
                        return None
                else:
                    buffer = frame.data
                return buffer, frame_rgb.shape, do_encoding
            else:
                print('Failed to capture an image')
        else:
            print('Failed to open the camera {}'.format(camera_id))
        return None


def process_set_camera_prop(msg):
    print("Set camera props: camera {} width {} height {} encoded {}".format(
        msg.camera_id, msg.frame_width, msg.frame_height, msg.do_encoding))
    set_camera_prop(msg.camera_id, msg.frame_width,
                    msg.frame_height, msg.fps, msg.do_encoding)


def process_capture_image(msg):
    # print("Capturing image: camera {}")
    img, shape, encoded = capture_image(msg.camera_id)
    response = SendImageMsg()
    response.set_img(msg.camera_id, img, shape[2], shape[1], shape[0], encoded)
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
    response.camera_id = msg.camera_id
    response.set_camera_prop(camera_props)
    return response


def process_move(chassis, msg):
    print("Moving left {}:{} right {}:{}".format(
        msg.left_speed, msg.left_dir, msg.right_speed, msg.right_dir))
    chassis.move(msg)


def process_stop(msg):
    print('Stopping...')
    for _, cam in cameras.items():
        cam.release()
    return StopMsg()


def process_message(msg, args):
    # print("Processing msg id : {}".format(msg.id()))
    result = {
        MessageId.CAPTURE_IMAGE: process_capture_image,
        MessageId.GET_CAMERA_LIST: process_get_camera_list,
        MessageId.MOVE: lambda msg: process_move(args, msg),
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
        chassis = Chassis()
        chassis.activate()

        client = Client(lambda msg: process_message(msg, chassis), get_msg_obj)
        client.init(host, port)

        capture_thread = threading.Thread(
            target=image_capture_thread_func, args=(client,))
        capture_thread.start()

        done = False
        while not done:
            done = not client.process_recv_message()

        chassis.dectivate()
        stop_event.set()
        capture_thread.join()
        client.close()


if __name__ == '__main__':
    main()
