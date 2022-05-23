"""The VB3YEET APRS messanger script
"""

import threading
from typing import List
from atentry import entry
import os
import logging
import aprs
import cabrillo
import datetime
import time

logger = logging.getLogger(__name__)

MESSAGE_RESPONSE = "Log updated TNX. More info at va3zza.com/yeet 73!"


def on_message(frame: aprs.Frame, connection: aprs.TCP, callsign: str) -> cabrillo.QSO:
    logger.info(f"Received message from: {frame.source.callsign}")

    # Respond with our message
    ssid = f"-{frame.source.ssid.decode()}" if frame.source.ssid != b'0' else ""
    connection.send(  # type: ignore
        f"{callsign}>APRS::{frame.source.callsign.decode()}{ssid}: {MESSAGE_RESPONSE}".encode())
    logger.info(f"Sent response to: {frame.source.callsign}")

    # Construct a QSO object for this message
    return cabrillo.QSO(
        144.39,
        'DG',
        datetime.datetime.now(),
        callsign,
        frame.source.callsign
    )


@entry
def main() -> int:

    # Setup logging
    logging.basicConfig(level=logging.INFO)

    # Get our login info from the environment and validate
    callsign = os.environ.get("CALLSIGN")
    passcode = os.environ.get("PASSCODE")
    log_file_path = os.environ.get("LOG_FILE_PATH", "./log.cbr")
    if not callsign or not passcode:
        logger.error("CALLSIGN or PASSCODE not set")
        return 1
    logger.info(f"Running with callsign {callsign}")

    # Open an APRS connection
    aprs_conn = aprs.TCP(callsign.encode(), passcode.encode())
    aprs_conn.start()

    # Create a QSO queue for periodic writing to disk
    qso_queue: List[cabrillo.QSO] = []

    # Link our callback fn to handle inbound messages
    threading.Thread(target=lambda: aprs_conn.receive(callback=lambda x: qso_queue.append(on_message(  # type: ignore
        x, aprs_conn, callsign)))).start()  # type: ignore


    # Wait for the connection to close
    while True:
        # Sleep for a while between log writes
        pass
        # try:
        #     time.sleep(10)
        # except KeyboardInterrupt:
        #     logger.info("Exiting")
        #     break

        # # Clone and clear the QSO queue
        # qso_queue_copy = qso_queue.copy()
        # qso_queue.clear()

        # # Inform
        # logger.info(f"Writing {len(qso_queue_copy)} QSOs to {log_file_path}")

        # # Load the existing log file if it exists
        # if os.path.exists(log_file_path):
        #     existing_log = cabrillo.parser.parse_log_file(log_file_path)
        # else:
        #     existing_log = cabrillo.Cabrillo()
        #     existing_log.callsign = callsign
        
        # # Extend the qso list
        # existing_log.qso.extend(qso_queue_copy)

        # # Write the log to disk
        # with open(log_file_path, "w") as f:
        #     existing_log.write(f)

    return 0
