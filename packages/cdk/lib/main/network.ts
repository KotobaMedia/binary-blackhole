import * as ec2 from "aws-cdk-lib/aws-ec2";
import { Construct } from "constructs";

export interface VPCProps {}

export class VPC extends Construct {
  vpc: ec2.Vpc;

  constructor(scope: Construct, id: string, props: VPCProps) {
    super(scope, id);

    this.vpc = new ec2.Vpc(this, "MainVPC", {
      // TODO: Dev/Prod should have different CIDR ranges
      ipAddresses: ec2.IpAddresses.cidr("10.100.0.0/16"),
      ipProtocol: ec2.IpProtocol.DUAL_STACK,
      maxAzs: 3,
      natGatewayProvider: ec2.NatProvider.gateway(),
      // Single NAT Gateway
      // TODO: migrate to something else. Maybe something that works with IPv6-only?
      // OpenAI requires IPv4 now.
      natGateways: 1,
      subnetConfiguration: [
        {
          cidrMask: 22,
          name: "Public",
          subnetType: ec2.SubnetType.PUBLIC,
        },
        {
          cidrMask: 22,
          name: "Private",
          subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
        },
        {
          cidrMask: 22,
          name: "Isolated",
          subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
        },
      ],
      gatewayEndpoints: {
        S3: {
          service: ec2.GatewayVpcEndpointAwsService.S3,
        },
        DynamoDB: {
          service: ec2.GatewayVpcEndpointAwsService.DYNAMODB,
        },
      },
      enableDnsHostnames: true,
      enableDnsSupport: true,
    });
  }
}
