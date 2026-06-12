import { RpcUtils } from "@/utils/RpcUtils";
import { JSX, useEffect, useRef, useState } from "react";
import { Button, Modal } from "react-bootstrap";

interface ConsoleOutputProps {
  onClose: () => void;
  onReady: () => void;
}

export function ConsoleOutput({
  onClose,
  onReady,
}: ConsoleOutputProps): JSX.Element {
  const [show, setShow] = useState(true);
  const [items, setItems] = useState<string[]>([]);
  const [finished, setFinished] = useState(false);

  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    RpcUtils.listen<string>("console_output", (msg) => {
      if (msg.trim().length > 0) {
        setItems((prev) => [...prev, msg]);
      } else {
        unlisten?.();
        setFinished(true);
      }
    }).then((fn) => {
      unlisten = fn;
      onReady();
    });
  }, []);

  // 👇 auto-scroll al final cuando cambian los items
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;

    el.scrollTop = el.scrollHeight;
  }, [items]);

  const onClosed = () => {
    setShow(false);
    setTimeout(() => {
      onClose();
    }, 200);
  };

  return (
    <Modal show={show} onHide={onClosed} backdrop="static" keyboard={false}>
      <Modal.Header>
        <Modal.Title>Process output</Modal.Title>
      </Modal.Header>

      <Modal.Body
        style={{
          display: "flex",
          flexDirection: "column",
          maxHeight: "60vh",
          overflow: "hidden",
        }}
      >
        <div
          ref={scrollRef}
          style={{
            flex: 1,
            overflowY: "auto",
            backgroundColor: "black",
            color: "white",
            padding: "5px",
          }}
        >
          {items.map((str, index) => (
            <p
              key={index}
              style={{ margin: 0, fontSize: "13px", height: "22px" }}
            >
              <pre style={{ margin: 0 }}>{str}</pre>
            </p>
          ))}
        </div>
      </Modal.Body>

      {finished && (
        <Modal.Footer>
          <Button variant="secondary" onClick={onClosed}>
            Close
          </Button>
        </Modal.Footer>
      )}
    </Modal>
  );
}
