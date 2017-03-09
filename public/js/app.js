(function() {
  var myApp = angular.module('progressServer', ['ngResource', 'ngMaterial', 'ui.router']);

  myApp.config(function($stateProvider, $urlRouterProvider) {
    $urlRouterProvider.otherwise('/home');
    $stateProvider
      .state('home', {
        url: '/home',
        templateUrl: '/static/html/home.html',
        controller: 'homeController',
        controllerAs: 'home',
      })
      .state('searchProcedure', {
        url: '/searchProcedure/:contents',
        templateUrl: '/static/html/searchProcedure.html',
        controller: 'searchProcedureController',
        controllerAs: 'search',
      })
      .state('searchInnerProcedure', {
        url: '/searchInnerProcedure/:fileName/:innerName',
        templateUrl: '/static/html/searchInnerProcedure.html',
        controller: 'searchInnerProcedureController',
        controllerAs: 'search',
      })
      .state('program', {
        url: '/program/:name',
        templateUrl: '/static/html/program.html',
        controller: 'programController',
        controllerAs: 'program',
      });
  });
}());
