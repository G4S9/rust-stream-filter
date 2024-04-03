from cdktf import Testing as T
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
from cdktf_cdktf_provider_aws.s3_control_object_lambda_access_point import S3ControlObjectLambdaAccessPoint

from main import AppStack


class TestMain:
    stack = AppStack(T.app(), "CodeChallenge")
    synthesized = T.synth(stack)

    def test_should_contain_aws_provider(self):
        assert T.to_have_provider(self.synthesized, AwsProvider.TF_RESOURCE_TYPE)

    def test_should_contain_s3_bucket(self):
        assert T.to_have_resource(self.synthesized, S3Bucket.TF_RESOURCE_TYPE)

    def test_should_contain_s3_access_point(self):
        assert T.to_have_resource(self.synthesized, S3AccessPoint.TF_RESOURCE_TYPE)

    def test_should_contain_api_gateway_rest_api(self):
        assert T.to_have_resource(self.synthesized, ApiGatewayRestApi.TF_RESOURCE_TYPE)

    def test_should_contain_api_gateway_resource(self):
        assert T.to_have_resource(self.synthesized, ApiGatewayResource.TF_RESOURCE_TYPE)

    def test_should_contain_api_gateway_method(self):
        assert T.to_have_resource(self.synthesized, ApiGatewayMethod.TF_RESOURCE_TYPE)

    def test_should_contain_api_gateway_integration(self):
        assert T.to_have_resource(self.synthesized, ApiGatewayIntegration.TF_RESOURCE_TYPE)

    def test_should_contain_lambda_function(self):
        assert T.to_have_resource(self.synthesized, LambdaFunction.TF_RESOURCE_TYPE)

    def test_should_contain_lambda_permission(self):
        assert T.to_have_resource(self.synthesized, LambdaPermission.TF_RESOURCE_TYPE)

    def test_should_contain_iam_role(self):
        assert T.to_have_resource(self.synthesized, IamRole.TF_RESOURCE_TYPE)

    def test_should_contain_iam_role_policy_attachment(self):
        assert T.to_have_resource(self.synthesized, IamRolePolicyAttachment.TF_RESOURCE_TYPE)

    def test_should_contain_iam_role_policy(self):
        assert T.to_have_resource(self.synthesized, IamRolePolicy.TF_RESOURCE_TYPE)

    def test_should_contain_api_gateway_deployment(self):
        assert T.to_have_resource(self.synthesized, ApiGatewayDeployment.TF_RESOURCE_TYPE)

    def test_should_contain_api_gateway_stage(self):
        assert T.to_have_resource(self.synthesized, ApiGatewayStage.TF_RESOURCE_TYPE)

    def test_should_contain_s3_control_object_lambda_access_point(self):
        assert T.to_have_resource(self.synthesized, S3ControlObjectLambdaAccessPoint.TF_RESOURCE_TYPE)
