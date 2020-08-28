package com.live2o3.example;

import androidx.appcompat.app.AppCompatActivity;

import android.os.Bundle;

import com.live2o3.Result;
import com.live2o3.chua.Chua;

public class MainActivity extends AppCompatActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Chua.test();
        Result<String> result = Chua.upload("wtf", "haha", 1234567, 0);
        System.out.println("Chua: " + result);
    }
}