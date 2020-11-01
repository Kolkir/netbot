import threading
import time
import os
is_arm_platform = not os.uname().machine == 'x86_64'
if is_arm_platform:
    import OPi.GPIO as GPIO


def init_control_pins(control_pins):
    for pin in control_pins:
        GPIO.setup(pin, GPIO.OUT)
        GPIO.output(pin, 0)


def rotate_wheels_thread_func(platform):
    while platform.is_active():
        platform.rotate_left_wheel()
        platform.rotate_right_wheel()
        time.sleep(0.001)


class Chassis:
    def __init__(self):
        self.left_control_pins = [12, 11, 6, 1]
        self.right_control_pins = [3, 15, 16, 14]
        self.halfstep_seq = [
            [1, 0, 0, 0],
            [1, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 1, 0],
            [0, 0, 1, 0],
            [0, 0, 1, 1],
            [0, 0, 0, 1],
            [1, 0, 0, 1]
        ]
        if is_arm_platform:
            GPIO.setboard(GPIO.ZEROPLUS)
            GPIO.setmode(GPIO.BOARD)
            init_control_pins(self.left_control_pins)
            init_control_pins(self.right_control_pins)

        self.stop_event = threading.Event()
        self.left_wheel_lock = threading.RLock()
        self.do_left_wheel_rotate = False
        self.right_wheel_lock = threading.RLock()
        self.do_right_wheel_rotate = False
        self.wheels_thread = threading.Thread(
            target=rotate_wheels_thread_func, args=(self,))

    def activate(self):
        self.wheels_thread.start()

    def is_active(self):
        return not self.stop_event.is_set()

    def dectivate(self):
        self.stop_event.set()
        self.wheels_thread.join()
        if is_arm_platform:
            GPIO.cleanup()

    def start_left_wheel_rotation(self):
        with self.left_wheel_lock:
            self.do_left_wheel_rotate = True

    def stop_left_wheel_rotation(self):
        with self.left_wheel_lock:
            self.do_left_wheel_rotate = False

    def start_right_wheel_rotation(self):
        with self.right_wheel_lock:
            self.do_right_wheel_rotate = True

    def stop_right_wheel_rotation(self):
        with self.right_wheel_lock:
            self.do_right_wheel_rotate = False

    def rotate_left_wheel(self):
        do_rotate = False
        with self.left_wheel_lock:
            do_rotate = self.do_left_wheel_rotate
        if do_rotate:
            self.rotate_motor(self.left_control_pins)

    def rotate_right_wheel(self):
        do_rotate = False
        with self.right_wheel_lock:
            do_rotate = self.do_right_wheel_rotate
        if do_rotate:
            self.rotate_motor(self.right_control_pins)

    def rotate_motor(self, control_pins):
        if is_arm_platform:
            for halfstep in range(512):
                for halfstep in range(8):
                    for pin in range(4):
                        GPIO.output(control_pins[pin],
                                    self.halfstep_seq[halfstep][pin])
                    time.sleep(0.001)
        else:
            print("moving")
