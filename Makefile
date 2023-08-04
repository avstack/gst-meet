.PHONY: help

AWS_ACCOUNT_ID = 718762496685
APP_NAME = gstreamer-pipeline
REGION = ap-south-1
NAMESPACE = sariska


build-release:
			docker build -t ${AWS_ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${NAMESPACE}/$(APP_NAME):latest .


push-release:
			docker push ${AWS_ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${NAMESPACE}/$(APP_NAME):latest


deploy-release:
			kubectl kustomize ./k8s | kubectl apply -k ./k8s

deploy: build-release push-release deploy-release

