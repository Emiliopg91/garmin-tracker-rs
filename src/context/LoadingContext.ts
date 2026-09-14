import { createContext } from "react";

interface LoadingContextType {
  loading: boolean;
  startLoading: () => void;
  finishLoading: () => void;
}

const defaultValue: LoadingContextType = {
  loading: false,
  startLoading: () => {
    /* empty */
  },
  finishLoading: () => {
    /* empty */
  },
};

export const LoadingContext = createContext(defaultValue);
