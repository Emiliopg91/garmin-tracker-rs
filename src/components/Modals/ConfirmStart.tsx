import { ChangeEvent, JSX, useState } from "react";
import { Button, Modal } from "react-bootstrap";

const GIT_BRANCH_REGEX =
  // eslint-disable-next-line no-useless-escape
  /^(?!.*\/\/)(?!.*\.\.)(?!.*@\{)(?!.*\.$)[a-zA-Z0-9\/_\.\-]*$/;

interface ConfirmStartProps {
  onClose: () => void;
  onConfirm: (name: string) => void;
}

export function ConfirmStart({
  onClose,
  onConfirm,
}: ConfirmStartProps): JSX.Element {
  const [show, setShow] = useState(true);
  const [name, setSetName] = useState("");
  const [enabled, setEnabled] = useState(false);

  const onConfirmed = () => {
    setShow(false);
    setTimeout(() => {
      onConfirm(name);
    }, 200);
  };

  const onClosed = () => {
    setShow(false);
    setTimeout(() => {
      onClose();
    }, 200);
  };

  const onInputChange = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;

    setSetName(value);

    const isValid = value.length > 3 && GIT_BRANCH_REGEX.test(value);

    setEnabled(isValid);
  };

  return (
    <Modal show={show} onHide={onClosed} backdrop="static" keyboard={false}>
      <Modal.Header closeButton>
        <Modal.Title>Start flow</Modal.Title>
      </Modal.Header>

      <Modal.Body>
        <input
          value={name}
          placeholder="Input name"
          style={{ width: "100%", padding: "5px" }}
          onChange={onInputChange}
        />
      </Modal.Body>

      <Modal.Footer>
        <Button variant="primary" onClick={onConfirmed} disabled={!enabled}>
          Save
        </Button>
        <Button variant="secondary" onClick={onClosed}>
          Close
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
