(function() {
    var myApp = angular.module('progressServer', ['ngResource', 'ui.router']);

    myApp.config(function($stateProvider, $urlRouterProvider) {
        $urlRouterProvider.otherwise('/home');
        $stateProvider
            .state('home', {
                url: '/home',
                templateUrl: '/static/html/home.html',
            })
            .state('program', {
                url: '/program/:name',
                templateUrl: '/static/html/program.html',
                controller: 'programController',
                controllerAs: 'program'
            });
    });
}());
