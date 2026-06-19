/* eslint-disable */

//Auto generated file, do not edit manually

import { invoke, InvokeArgs } from "@tauri-apps/api/core";

export class RustBridge {

	private static inner_invoke<R>(method: string, payload?: InvokeArgs): Promise<R> {
		return invoke(method, payload);
	}
}