#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { BBHStage } from '../lib/stage';

const app = new cdk.App();

if (!process.env.CDK_DEV_ACCOUNT_ID) {
  throw new Error('CDK_DEV_ACCOUNT_ID must be defined');
}

new BBHStage(app, 'Dev', {
  env: {
    account: process.env.CDK_DEV_ACCOUNT_ID,
    region: 'ap-northeast-1'
  }
});
