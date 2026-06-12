import { JSX, useState } from "react";
import { Button, Modal } from "react-bootstrap";

interface ConfirmFinishProps {
  name: string;
  onClose: () => void;
  onConfirm: () => void;
}

export function ConfirmFinish({
  name: branch,
  onClose,
  onConfirm,
}: ConfirmFinishProps): JSX.Element {
  const [show, setShow] = useState(true);

  const onConfirmed = () => {
    setShow(false);
    setTimeout(() => {
      onConfirm();
    }, 200);
  };

  const onClosed = () => {
    setShow(false);
    setTimeout(() => {
      onClose();
    }, 200);
  };

  return (
    <Modal show={show} onHide={onClosed} backdrop="static" keyboard={false}>
      <Modal.Header closeButton>
        <Modal.Title>Finish</Modal.Title>
      </Modal.Header>

      <Modal.Body>
        <p>Are you sure to finish {branch}?</p>
      </Modal.Body>

      <Modal.Footer>
        <Button variant="secondary" onClick={onClosed}>
          No
        </Button>

        <Button variant="danger" onClick={onConfirmed}>
          Yes
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
