import multiprocessing
import time
import os
import sys
is_arm_platform = not os.uname().machine == 'x86_64'
if is_arm_platform:
    import OPi.GPIO as GPIO


def init_control_pins(control_pins):
    GPIO.setup(control_pins, GPIO.OUT)
    GPIO.output(control_pins, 0)


def rotate_wheels_process_func(events):
    sys.stdout = open(str(os.getpid()) + ".out", "a")
    sys.stderr = open(str(os.getpid()) + "_error.out", "a")
    print('Chassis init')
    chassis = ChassisProcess(events)
    while chassis.is_active():
        chassis.update_wheels_config()
        chassis.rotate_wheels()
        time.sleep(0.001)
    chassis.dectivate()
    print("Chassis deactivated")


class ChassisProcess:
    def __init__(self, events):
        self.left_control_pins = [3, 5, 7, 16]
        self.right_control_pins = [15, 19, 21, 23]
        self.right_halfstep_seq = [
            [1, 0, 0, 0],
            [1, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 1, 0],
            [0, 0, 1, 0],
            [0, 0, 1, 1],
            [0, 0, 0, 1],
            [1, 0, 0, 1]
        ]
        self.right_step_index = 0
        self.left_halfstep_seq = self.right_halfstep_seq[::-1]
        self.left_step_index = 0
        self.left_wheel_enabled = False
        self.right_wheel_enabled = False

        if is_arm_platform:
            # GPIO.setboard(GPIO.ZEROPLUS)
            GPIO.setmode(GPIO.BOARD)
            init_control_pins(self.left_control_pins)
            init_control_pins(self.right_control_pins)

        self.stop_event = events[0]
        self.enable_left_wheel_event = events[1]
        self.enable_right_wheel_event = events[2]
        self.left_wheel_backward_event = events[3]
        self.right_wheel_backward_event = events[4]

    def is_active(self):
        return not self.stop_event.is_set()

    def dectivate(self):
        if is_arm_platform:
            GPIO.cleanup()

    def update_wheels_config(self):
        if self.enable_left_wheel_event.is_set():
            self.left_wheel_enabled = True
        else:
            self.left_wheel_enabled = False

        if self.enable_right_wheel_event.is_set():
            self.right_wheel_enabled = True
        else:
            self.right_wheel_enabled = False

    def rotate_wheels(self):
        print("moving")
        control_pins = []
        pins_values = []

        if self.left_wheel_enabled:
            control_pins += self.left_control_pins
            for pin in range(4):
                pins_values.append(
                    self.left_halfstep_seq[self.right_step_index][pin])
            self.left_step_index += 1
            if self.left_step_index > 7:
                self.left_step_index = 0

        if self.right_wheel_enabled:
            control_pins += self.right_control_pins
            for pin in range(4):
                pins_values.append(
                    self.right_halfstep_seq[self.right_step_index][pin])
            self.right_step_index += 1
            if self.right_step_index > 7:
                self.right_step_index = 0

        if is_arm_platform and len(control_pins) > 0:
            GPIO.output(control_pins, pins_values)

    def rotate_right_motor(self):
        if is_arm_platform:
            for halfstep in range(512):
                for halfstep in range(8):
                    for pin in range(4):
                        GPIO.output(self.right_control_pins[pin],
                                    self.right_halfstep_seq[halfstep][pin])
                    time.sleep(0.001)
        else:
            print("moving")


class Chassis:
    def __init__(self):
        self.stop_event = multiprocessing.Event()
        self.enable_left_wheel_event = multiprocessing.Event()
        self.enable_right_wheel_event = multiprocessing.Event()
        self.left_wheel_backward_event = multiprocessing.Event()
        self.right_wheel_backward_event = multiprocessing.Event()
        events = [self.stop_event,
                  self.enable_left_wheel_event,
                  self.enable_right_wheel_event,
                  self.left_wheel_backward_event,
                  self.right_wheel_backward_event,
                  ]
        self.wheels_process = multiprocessing.Process(
            name='wheels_process', target=rotate_wheels_process_func, args=(events,))

    def activate(self):
        self.wheels_process.start()

    def dectivate(self):
        self.stop_event.set()
        self.wheels_process.join()

    def move(self, msg):
        if msg.left_speed != 0:
            self.enable_left_wheel_event.set()
        else:
            self.enable_left_wheel_event.clear()

        if msg.left_dir == 0:
            self.left_wheel_backward_event.set()
        else:
            self.left_wheel_backward_event.clear()

        if msg.right_speed != 0:
            self.enable_right_wheel_event.set()
        else:
            self.enable_right_wheel_event.clear()

        if msg.right_dir == 0:
            self.right_wheel_backward_event.set()
        else:
            self.right_wheel_backward_event.clear()
