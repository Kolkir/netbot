import OPi.GPIO as GPIO
import time

def init_pins(pins):
    for pin in pins:
        print('Configuring pin {}'.format(pin))
        GPIO.setup(pin, GPIO.OUT, initial=GPIO.LOW)


GPIO.cleanup()
GPIO.setmode(GPIO.BOARD)
print('Mode is board {}'.format(GPIO.getmode()==GPIO.BOARD))

left_control_pins = [3, 5, 7, 16]
right_control_pins = [15, 19, 21, 23]

init_pins(left_control_pins)
init_pins(right_control_pins)

GPIO.cleanup()
