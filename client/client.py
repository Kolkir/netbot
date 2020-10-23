
import socket as sock
from threading import RLock
from message import MessageId, HelloMsg, StopMsg


class Client:

    def __init__(self, process_message_func, get_message_obj_func):
        self.process_message = process_message_func
        self.get_message_obj = get_message_obj_func
        self.socket = None
        self.guard = RLock()

    def send_msg(self, msg):
        with self.guard:
            # send header
            header = bytearray()
            header.extend(msg.id().to_bytes(1, byteorder="big"))
            header.extend(msg.size().to_bytes(4, byteorder="big"))
            self.socket.sendall(header)
            # send body
            bytes = msg.to_bytes()
            if bytes:
                self.socket.sendall(bytes)

    def recv_msg(self):
        # recv header`
        id_size_data = self.socket.recv(5)
        id = id_size_data[0]
        size = int.from_bytes(id_size_data[1:5], byteorder='big')
        # recv body
        data = self.socket.recv(size)
        msg = self.get_message_obj(id)
        msg.from_bytes(data)
        return msg

    def close(self):
        self.socket.close()

    def init(self, host, port):
        self.socket = sock.socket(sock.AF_INET, sock.SOCK_STREAM)
        self.socket.connect((host, port))
        # handshakes
        hello_msg = HelloMsg()
        self.send_msg(hello_msg)
        hello_msg = self.recv_msg()
        if MessageId.HELLO == hello_msg.id():
            print('Handshake completed')
        else:
            print('Handshake failed, msg_id={}'.format(hello_msg.id()))
            exit(1)

    def process_recv_message(self):
        msg = self.recv_msg()
        # print("Got msg_id {}".format(msg.id()))
        response = self.process_message(msg)
        if type(response) is StopMsg:
            return False
        if response:
            self.send_msg(response)
        return True
