
import socket as sock
from message import MessageId, HelloMsg


class Client:

    def __init__(self, process_message_func, get_message_obj_func):
        self.process_message = process_message_func
        self.get_message_obj = get_message_obj_func

    def send_msg(self, socket, msg):
        # send header
        header = bytearray()
        header.extend(msg.id().to_bytes(1, byteorder="big"))
        header.extend(msg.size().to_bytes(4, byteorder="big"))
        socket.sendall(header)
        # send body
        bytes = msg.to_bytes()
        if bytes:
            socket.sendall(bytes)

    def recv_msg(self, socket):
        # recv header`
        id_size_data = socket.recv(5)
        id = id_size_data[0]
        size = int.from_bytes(id_size_data[1:5], byteorder='big')
        # recv body
        data = socket.recv(size)
        msg = self.get_message_obj(id)
        msg.from_bytes(data)
        return msg

    def run(self, host, port):
        with sock.socket(sock.AF_INET, sock.SOCK_STREAM) as s:
            s.connect((host, port))
            # handshakes
            hello_msg = HelloMsg()
            self.send_msg(s, hello_msg)
            hello_msg = self.recv_msg(s)
            if MessageId.HELLO == hello_msg.id():
                print('Handshake completed')
                done = False
                while not done:
                    msg = self.recv_msg(s)
                    print("Got msg_id {}".format(msg.id()))
                    response = self.process_message(msg)
                    self.send_msg(s, response)
                print('Connection closed')
            else:
                print('Handshake failed, msg_id={}'.format(hello_msg.id()))
                exit(1)
