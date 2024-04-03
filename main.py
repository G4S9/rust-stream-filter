#!/usr/bin/env python
import hashlib
import json
import os

from pathlib import Path

import cdktf
from constructs import Construct
from cdktf import App, TerraformStack, S3Backend, TerraformAsset, AssetType
from cdktf_cdktf_provider_aws.provider import AwsProvider
from cdktf_cdktf_provider_aws.s3_bucket import S3Bucket
from cdktf_cdktf_provider_aws.s3_access_point import S3AccessPoint
from cdktf_cdktf_provider_aws.api_gateway_rest_api import ApiGatewayRestApi
from cdktf_cdktf_provider_aws.api_gateway_resource import ApiGatewayResource
from cdktf_cdktf_provider_aws.api_gateway_method import ApiGatewayMethod
from cdktf_cdktf_provider_aws.api_gateway_integration import ApiGatewayIntegration
from cdktf_cdktf_provider_aws.lambda_function import LambdaFunction
from cdktf_cdktf_provider_aws.lambda_permission import LambdaPermission
from cdktf_cdktf_provider_aws.iam_role import IamRole
from cdktf_cdktf_provider_aws.iam_role_policy_attachment import IamRolePolicyAttachment
from cdktf_cdktf_provider_aws.iam_role_policy import IamRolePolicy
from cdktf_cdktf_provider_aws.api_gateway_deployment import ApiGatewayDeployment
from cdktf_cdktf_provider_aws.api_gateway_stage import ApiGatewayStage
from cdktf_cdktf_provider_aws.api_gateway_method_settings import ApiGatewayMethodSettings
from cdktf_cdktf_provider_aws.s3_control_object_lambda_access_point import S3ControlObjectLambdaAccessPoint

BASE_DIR = Path(__file__).parent
ENV = os.getenv('ENV') or 'dev'
REGION = os.getenv('AWS_REGION') or 'eu-central-1'
STATE_BUCKET = os.getenv('STATE_BUCKET') or 'g4s9-terraform-state-bucket'
STATE_PREFIX = ENV
STATE_LOCK_TABLE = os.getenv('LOCK_TABLE') or 'g4s9-terraform-lock-table'
ENABLE_CACHING = bool(os.getenv("ENABLE_CACHING"))


class AppStack(TerraformStack):
    def __init__(self, scope: Construct, id: str):
        super().__init__(scope, id)

        AwsProvider(self, 'AWS', region=REGION)

        S3Backend(
            self,
            bucket=STATE_BUCKET,
            key=f"{self.node.try_get_context('projectName')}/{STATE_PREFIX}/terraform.tfstate",
            encrypt=True,
            region=REGION,
            dynamodb_table=STATE_LOCK_TABLE,
        )

        self.__s3_resources()
        self.__object_lambda_resources()
        self.__api_lambda_resources()
        self.__api_gateway_resources()

    def __s3_resources(self):
        self.s3_bucket = S3Bucket(self, 'S3Bucket', force_destroy=True)

        self.s3_access_point = S3AccessPoint(
            self,
            "S3AccessPoint",
            bucket=self.s3_bucket.id,
            # value is mandatory and must be globally unique...
            # must adhere to DNS naming rules as well
            name=f"{self.s3_bucket.id}-sap"
        )

    def __object_lambda_resources(self):
        self.object_lambda_execution_role = IamRole(
            self, "ObjectLambdaExecutionRole",
            assume_role_policy="""{
                "Version": "2012-10-17",
                "Statement": [{
                  "Action": "sts:AssumeRole",
                  "Principal": {
                    "Service": "lambda.amazonaws.com"
                  },
                  "Effect": "Allow",
                  "Sid": ""
                }]
              }"""
        )

        IamRolePolicyAttachment(
            self, "ObjectLambdaExecutionRolePolicyAttachment",
            role=self.object_lambda_execution_role.name,
            policy_arn="arn:aws:iam::aws:policy/service-role/AmazonS3ObjectLambdaExecutionRolePolicy"
        )

        self.object_lambda_code = TerraformAsset(
            self, "ObjectLambdaCode", path=str(BASE_DIR / "src/object_lambda/target/lambda/object_lambda"),
            type=AssetType.ARCHIVE
        )

        self.object_lambda_function = LambdaFunction(
            self, "ObjectLambdaFunction",
            function_name="ObjectLambdaFunction",
            # not used for the provided runtime, but required by syntax
            handler="index.handler",
            runtime="provided.al2023",
            role=self.object_lambda_execution_role.arn,
            filename=self.object_lambda_code.path,
            source_code_hash=cdktf.Fn.filebase64sha256(self.object_lambda_code.path),
            # 1749 means also 1 vCPU
            memory_size=1769,
            timeout=60
        )

        self.object_lambda_access_point = S3ControlObjectLambdaAccessPoint(
            self,
            "S3ControlObjectLambdaAccessPoint",
            # value is mandatory and must be globally unique...
            # must adhere to DNS naming rules as well
            name=f"{self.s3_bucket.id}-lap",
            configuration={
                "supporting_access_point": self.s3_access_point.arn,
                "transformation_configuration": [{
                    "actions": ["GetObject"],
                    "contentTransformation": {"awsLambda": {"functionArn": self.object_lambda_function.arn}}
                }]
            }
        )

    def __api_lambda_resources(self):
        self.api_lambda_execution_role = IamRole(
            self, "ApiLambdaExecutionRole",
            assume_role_policy="""{
                "Version": "2012-10-17",
                "Statement": [{
                  "Action": "sts:AssumeRole",
                  "Principal": {
                    "Service": "lambda.amazonaws.com"
                  },
                  "Effect": "Allow",
                  "Sid": ""
                }]
              }"""
        )

        IamRolePolicyAttachment(
            self, "ApiLambdaExecutionRolePolicyAttachment",
            role=self.api_lambda_execution_role.name,
            policy_arn="arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
        )

        IamRolePolicy(
            self, "ApiLambdaExecutionRoleInlinePolicy",
            role=self.api_lambda_execution_role.id,
            policy=f"""{{
                "Version": "2012-10-17",
                "Statement": [{{
                    "Effect": "Allow",
                    "Action": [
                        "s3:ListBucket"
                    ],
                    "Resource": "arn:aws:s3:::{self.s3_bucket.id}"
                }},{{
                    "Effect": "Allow",
                    "Action": [
                        "s3:PutObject",
                        "s3:GetObject",
                        "s3:DeleteObject"
                    ],
                    "Resource": "arn:aws:s3:::{self.s3_bucket.id}/*"
                }},{{
                    "Effect": "Allow",
                    "Action": [
                        "s3-object-lambda:GetObject"
                    ],
                    "Resource": "{self.object_lambda_access_point.arn}"
                }},{{
                    "Effect": "Allow",
                    "Action": [
                        "lambda:InvokeFunction"
                    ],
                    "Resource": "{self.object_lambda_function.arn}"
                }},{{
                      "Action": [
                        "s3:GetObject"
                      ],
                      "Effect": "Allow",
                      "Resource": "{self.s3_access_point.arn}/*"
                }}]
            }}"""
        )

        self.api_lambda_code = TerraformAsset(
            self, "ApiLambdaCode", path=str(BASE_DIR / "src/api_lambda/target/lambda/api_lambda"),
            type=AssetType.ARCHIVE
        )

        self.api_lambda_function = LambdaFunction(
            self, "ApiLambdaFunction",
            function_name="ApiLambdaFunction",
            # not used for the provided runtime, but required by syntax
            handler="index.handler",
            runtime="provided.al2023",
            role=self.api_lambda_execution_role.arn,
            filename=self.api_lambda_code.path,
            source_code_hash=cdktf.Fn.filebase64sha256(self.api_lambda_code.path),
            environment={"variables": {
                "BUCKET_NAME": self.s3_bucket.id,
                "OBJECT_LAMBDA_ENDPOINT_ARN": self.object_lambda_access_point.arn
            }}
        )

    def __api_gateway_resources(self):
        self.api_gateway = ApiGatewayRestApi(
            self, "ApiGateway",
            name="PhoneNumberAPI",
            description="API for managing phone numbers"
        )

        self.phone_numbers_resource = ApiGatewayResource(
            self, "PhoneNumbersResource",
            rest_api_id=self.api_gateway.id,
            parent_id=self.api_gateway.root_resource_id,
            path_part="phonenumbers"
        )

        self.phone_numbers_post = ApiGatewayMethod(
            self, "PhoneNumbersPost",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_numbers_resource.id,
            http_method="POST",
            authorization="NONE"
        )

        self.phone_numbers_post_integration = ApiGatewayIntegration(
            self,
            "PhoneNumbersPostIntegration",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_numbers_resource.id,
            http_method=self.phone_numbers_post.http_method,
            integration_http_method="POST",
            type="AWS_PROXY",
            uri=self.api_lambda_function.invoke_arn
        )

        self.phone_numbers_get = ApiGatewayMethod(
            self, "PhoneNumbersGet",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_numbers_resource.id,
            http_method="GET",
            authorization="NONE"
        )

        self.phone_numbers_get_integration = ApiGatewayIntegration(
            self,
            "PhoneNumbersGetIntegration",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_numbers_resource.id,
            http_method=self.phone_numbers_get.http_method,
            integration_http_method="POST",
            type="AWS_PROXY",
            uri=self.api_lambda_function.invoke_arn
        )

        self.phone_number_by_id_resource = ApiGatewayResource(
            self, "PhoneNumberByIdResource",
            rest_api_id=self.api_gateway.id,
            parent_id=self.phone_numbers_resource.id,
            path_part="{id}"
        )

        self.phone_number_get_by_id = ApiGatewayMethod(
            self, "PhoneNumberGetById",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_number_by_id_resource.id,
            http_method="GET",
            authorization="NONE"
        )

        self.phone_number_get_by_id_integration = ApiGatewayIntegration(
            self,
            "PhoneNumberGetByIdIntegration",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_number_by_id_resource.id,
            http_method=self.phone_number_get_by_id.http_method,
            integration_http_method="POST",
            type="AWS_PROXY",
            uri=self.api_lambda_function.invoke_arn
        )

        self.phone_number_delete_by_id = ApiGatewayMethod(
            self, "PhoneNumberDeleteById",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_number_by_id_resource.id,
            http_method="DELETE",
            authorization="NONE"
        )

        self.phone_number_delete_by_id_integration = ApiGatewayIntegration(
            self,
            "PhoneNumberDeleteByIdIntegration",
            rest_api_id=self.api_gateway.id,
            resource_id=self.phone_number_by_id_resource.id,
            http_method=self.phone_number_delete_by_id.http_method,
            integration_http_method="POST",
            type="AWS_PROXY",
            uri=self.api_lambda_function.invoke_arn
        )

        LambdaPermission(
            self, "ApiLambdaInvokePermission",
            action="lambda:InvokeFunction",
            function_name=self.api_lambda_function.function_name,
            principal="apigateway.amazonaws.com",
            source_arn=f"{self.api_gateway.execution_arn}/*/*/*"
        )

        self.api_gateway_deployment = ApiGatewayDeployment(
            self,
            "ApiGatewayDeployment",
            rest_api_id=self.api_gateway.id,
            triggers={
                "redeployment": hashlib.sha1(
                    json.dumps(
                        [
                            self.phone_numbers_resource.id,
                            self.phone_number_by_id_resource.id,
                            self.phone_numbers_get.id,
                            self.phone_numbers_get_integration.id,
                            self.phone_numbers_post.id,
                            self.phone_numbers_post_integration.id,
                            self.phone_number_get_by_id.id,
                            self.phone_number_get_by_id_integration.id,
                            self.phone_number_delete_by_id.id,
                            self.phone_number_delete_by_id_integration.id
                        ]
                    ).encode("utf-8")
                ).hexdigest()
            },
            lifecycle={
                "create_before_destroy": True,
            },
            depends_on=[
                self.phone_numbers_post,
                self.phone_numbers_get,
                self.phone_number_get_by_id,
                self.phone_number_delete_by_id,
                self.phone_numbers_post_integration,
                self.phone_numbers_get_integration,
                self.phone_number_get_by_id_integration,
                self.phone_number_delete_by_id_integration
            ]
        )

        self.api_gateway_stage = ApiGatewayStage(
            self,
            "ApiGatewayStage",
            rest_api_id=self.api_gateway.id,
            deployment_id=self.api_gateway_deployment.id,
            stage_name=ENV,
        )

        if ENABLE_CACHING:
            self.method_settings = ApiGatewayMethodSettings(
                self, "ApiGatewayMethodSettings",
                rest_api_id=self.api_gateway.id,
                stage_name=self.api_gateway_stage.stage_name,
                method_path="phonenumbers/{id}/GET",
                settings={
                    "caching_enabled": True,
                    "cache_data_encrypted": True,
                    "cache_ttl_in_seconds": 3600,
                    "require_authorization_for_cache_control": True,
                    "throttling_burst_limit": 5,
                    "throttling_rate_limit": 10
                }
            )

            self.api_gateway_stage.method_settings = [self.method_settings]

            self.api_gateway_stage.cache_cluster_enabled = True
            self.api_gateway_stage.cache_cluster_size = "0.5"


app = App()

AppStack(app, "CodeChallenge")

app.synth()
