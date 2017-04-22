Minimal copy (only copies are to/from the tokio encode/decoder buffer object) open pixel control rust library.

Example OPC server:

```
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();

    // send the message from the server to another thread...
    let (send, recv) = futures::sync::mpsc::channel(0);

    let server = listener.incoming().for_each(|(stream, _)| {
        let (_, stream) = stream.framed(opc::OPCCodec).split();
        // using this and not 'handle.spawn(...);Ok(())' should guarantee one connection at at time.
        send.clone().sink_map_err(|_| ()).send_all(stream.map_err(|_| ())).then(|_|Ok(()))
    });
```