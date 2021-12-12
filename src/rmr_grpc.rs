#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerDescription {
    #[prost(bytes = "vec", tag = "1")]
    pub uuid: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskDescription {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(enumeration = "TaskType", tag = "2")]
    pub task_type: i32,
    #[prost(uint32, tag = "3")]
    pub n: u32,
    #[prost(string, repeated, tag = "4")]
    pub files: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CurrentTask {
    #[prost(bytes = "vec", tag = "1")]
    pub uuid: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "2")]
    pub id: u32,
    #[prost(enumeration = "TaskType", tag = "3")]
    pub task_type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Acknowledge {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TaskType {
    Map = 0,
    Reduce = 1,
}
#[doc = r" Generated client implementations."]
pub mod coordinator_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct CoordinatorServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CoordinatorServiceClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> CoordinatorServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CoordinatorServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            CoordinatorServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn request_task(
            &mut self,
            request: impl tonic::IntoRequest<super::WorkerDescription>,
        ) -> Result<tonic::Response<super::TaskDescription>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/rmr_grpc.CoordinatorService/RequestTask");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn task_done(
            &mut self,
            request: impl tonic::IntoRequest<super::TaskDescription>,
        ) -> Result<tonic::Response<super::TaskDescription>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/rmr_grpc.CoordinatorService/TaskDone");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn task_failed(
            &mut self,
            request: impl tonic::IntoRequest<super::CurrentTask>,
        ) -> Result<tonic::Response<super::Acknowledge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/rmr_grpc.CoordinatorService/TaskFailed");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn notify_working(
            &mut self,
            request: impl tonic::IntoRequest<super::CurrentTask>,
        ) -> Result<tonic::Response<super::Acknowledge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/rmr_grpc.CoordinatorService/NotifyWorking");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod coordinator_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with CoordinatorServiceServer."]
    #[async_trait]
    pub trait CoordinatorService: Send + Sync + 'static {
        async fn request_task(
            &self,
            request: tonic::Request<super::WorkerDescription>,
        ) -> Result<tonic::Response<super::TaskDescription>, tonic::Status>;
        async fn task_done(
            &self,
            request: tonic::Request<super::TaskDescription>,
        ) -> Result<tonic::Response<super::TaskDescription>, tonic::Status>;
        async fn task_failed(
            &self,
            request: tonic::Request<super::CurrentTask>,
        ) -> Result<tonic::Response<super::Acknowledge>, tonic::Status>;
        async fn notify_working(
            &self,
            request: tonic::Request<super::CurrentTask>,
        ) -> Result<tonic::Response<super::Acknowledge>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct CoordinatorServiceServer<T: CoordinatorService> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: CoordinatorService> CoordinatorServiceServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        pub fn inner(&self) -> Arc<T> {
            self.inner.0.clone()
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CoordinatorServiceServer<T>
    where
        T: CoordinatorService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/rmr_grpc.CoordinatorService/RequestTask" => {
                    #[allow(non_camel_case_types)]
                    struct RequestTaskSvc<T: CoordinatorService>(pub Arc<T>);
                    impl<T: CoordinatorService>
                        tonic::server::UnaryService<super::WorkerDescription>
                        for RequestTaskSvc<T>
                    {
                        type Response = super::TaskDescription;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WorkerDescription>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).request_task(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RequestTaskSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rmr_grpc.CoordinatorService/TaskDone" => {
                    #[allow(non_camel_case_types)]
                    struct TaskDoneSvc<T: CoordinatorService>(pub Arc<T>);
                    impl<T: CoordinatorService> tonic::server::UnaryService<super::TaskDescription> for TaskDoneSvc<T> {
                        type Response = super::TaskDescription;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TaskDescription>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).task_done(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TaskDoneSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rmr_grpc.CoordinatorService/TaskFailed" => {
                    #[allow(non_camel_case_types)]
                    struct TaskFailedSvc<T: CoordinatorService>(pub Arc<T>);
                    impl<T: CoordinatorService> tonic::server::UnaryService<super::CurrentTask> for TaskFailedSvc<T> {
                        type Response = super::Acknowledge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CurrentTask>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).task_failed(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TaskFailedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rmr_grpc.CoordinatorService/NotifyWorking" => {
                    #[allow(non_camel_case_types)]
                    struct NotifyWorkingSvc<T: CoordinatorService>(pub Arc<T>);
                    impl<T: CoordinatorService> tonic::server::UnaryService<super::CurrentTask>
                        for NotifyWorkingSvc<T>
                    {
                        type Response = super::Acknowledge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CurrentTask>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).notify_working(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotifyWorkingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: CoordinatorService> Clone for CoordinatorServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: CoordinatorService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: CoordinatorService> tonic::transport::NamedService for CoordinatorServiceServer<T> {
        const NAME: &'static str = "rmr_grpc.CoordinatorService";
    }
}
